//! HTTP request paginator, supporting lazy traversal across large sets

use crate::http::error::HttpResult;
use sms_types::http::HttpPaginationOptions;

/// Call a function with an update `HttpPaginationOptions` for each batch request,
/// simplifying lazy access to large response sets such as messages etc.
pub struct HttpPaginator<T, F, Fut> {
    http_fn: F,
    pagination: HttpPaginationOptions,
    current_batch: Vec<T>,
    current_index: usize,
    has_more: bool,
    initial_limit: u64,
    _phantom: std::marker::PhantomData<Fut>,
}
impl<T, F, Fut> HttpPaginator<T, F, Fut>
where
    F: Fn(Option<HttpPaginationOptions>) -> Fut,
    Fut: Future<Output = HttpResult<Vec<T>>>,
{
    /// Create the paginator with the http batch generator.
    ///
    /// # Example
    /// ```text
    /// use sms_client::Client;
    /// use sms_client::config::ClientConfig;
    /// use sms_client::http::paginator::HttpPaginator;
    /// use sms_client::http::types::HttpPaginationOptions;
    ///
    /// let http = Client::new(ClientConfig::http_only("http://localhost:3000").with_auth("token!"))?.http_arc();
    /// let mut paginator = HttpPaginator::new(
    ///     move |pagination| {
    ///         let http = http.expect("Missing HTTP client configuration!").clone();
    ///         async move {
    ///             http.get_latest_numbers(pagination).await
    ///         }
    ///     },
    ///     HttpPaginationOptions::default()
    ///         .with_limit(10) // Do it in batches of 10.
    ///         .with_offset(10) // Skip the first 10 results.
    ///         .with_reverse(true) // Reverse the results set.
    /// );
    /// ```
    pub fn new(http_fn: F, pagination: HttpPaginationOptions) -> Self {
        let initial_limit = pagination.limit.unwrap_or(50);

        Self {
            http_fn,
            pagination,
            current_batch: Vec::new(),
            current_index: 0,
            has_more: true,
            initial_limit,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a paginator with default pagination settings.
    /// This starts at offset 0 with a limit of 50 per page.
    ///
    /// # Example
    /// ```text
    /// use sms_client::http;
    /// use sms_client::Client;
    /// use sms_client::config::ClientConfig;
    /// use sms_client::http::HttpClient;
    /// use sms_client::http::paginator::HttpPaginator;
    ///
    /// /// View all latest numbers, in a default paginator with a limit of 50 per chunk.
    /// async fn view_all_latest_numbers(http: HttpClient) {
    ///     let mut paginator = HttpPaginator::with_defaults(|pagination| {
    ///         http.get_latest_numbers(pagination)
    ///     });
    ///     while let Some(message) = paginator.next().await {
    ///         log::info!("{:?}", message);
    ///     }
    /// }
    /// ```
    pub fn with_defaults(http_fn: F) -> Self {
        Self::new(
            http_fn,
            HttpPaginationOptions::default()
                .with_limit(50)
                .with_offset(0),
        )
    }

    /// Fetch the next batch of items from the API.
    async fn fetch_next_batch(&mut self) -> HttpResult<bool> {
        let response = (self.http_fn)(Some(self.pagination)).await?;

        let received_count = response.len() as u64;
        self.has_more = received_count >= self.initial_limit;

        // If no more items have been received, we're definitely done.
        if received_count == 0 {
            self.has_more = false;
            return Ok(false);
        }

        self.current_batch = response;
        self.current_index = 0;

        // Update offset for next request.
        if let Some(current_offset) = self.pagination.offset {
            self.pagination.offset = Some(current_offset + received_count);
        } else {
            // If no offset was set initially, start from the received count
            self.pagination.offset = Some(received_count);
        }

        Ok(true)
    }

    /// Get the next item, automatically fetching next pages as needed.
    ///
    /// # Example
    /// ```text
    /// use sms_client::http::HttpClient;
    /// use sms_client::http::paginator::HttpPaginator;
    ///
    /// async fn get_delivery_reports(message_id: i64, http: HttpClient) {
    ///     let mut paginator = HttpPaginator::with_defaults(|pagination| {
    ///         http.get_delivery_reports(message_id, pagination)
    ///     }).await;
    ///
    ///     /// Iterate through ALL messages, with a page size of 50 (default).
    ///     while let Some(message) = paginator.next().await {
    ///         log::info!("{:?}", message);
    ///     }
    /// }
    /// ```
    pub async fn next(&mut self) -> Option<T> {
        if self.current_index >= self.current_batch.len() {
            // If there aren't any-more, then there is nothing to fetch next.
            if !self.has_more {
                return None;
            }

            match self.fetch_next_batch().await {
                Ok(true) => {}                     // Successfully fetched more data
                Ok(false) | Err(_) => return None, // No more data or error
            }
        }

        // Return the next item if available.
        if self.current_index < self.current_batch.len() {
            let item = self.current_batch.remove(0);
            Some(item)
        } else {
            None
        }
    }

    /// Collect all remaining items into a Vec.
    /// This continues to request batches until empty.
    pub async fn collect_all(mut self) -> HttpResult<Vec<T>> {
        let mut all_items = Vec::new();

        if self.current_batch.is_empty() && self.has_more {
            self.fetch_next_batch().await?;
        }

        while let Some(item) = self.next().await {
            all_items.push(item);
        }

        Ok(all_items)
    }

    /// Process items in chunks, calling the provided closure for each chunk.
    pub async fn take(mut self, n: usize) -> HttpResult<Vec<T>> {
        let mut items = Vec::with_capacity(n.min(100)); // Cap initial capacity

        for _ in 0..n {
            if let Some(item) = self.next().await {
                items.push(item);
            } else {
                break;
            }
        }

        Ok(items)
    }

    /// Process items in chunks, calling the provided closure for each chunk.
    ///
    /// # Example
    /// ```text
    /// use std::sync::Arc;
    /// use sms_client::http::HttpClient;
    /// use sms_client::http::paginator::HttpPaginator;
    /// use sms_client::http::types::HttpPaginationOptions;
    ///
    /// /// Read all messages from a phone number, in chunks of 10.
    /// async fn read_all_messages(phone_number: &str, http: Arc<HttpClient>) {
    ///     let paginator = HttpPaginator::with_defaults(|pagination| {
    ///         http.get_messages(phone_number, pagination)
    ///     }).await;
    ///
    ///     paginator.for_each_chuck(10, |batch| {
    ///         for message in batch {
    ///             log::info!("{:?}", message);
    ///         }
    ///     }).await?;
    /// }
    /// ```
    pub async fn for_each_chuck<C>(mut self, chunk_size: usize, mut chunk_fn: C) -> HttpResult<()>
    where
        C: FnMut(&[T]) -> HttpResult<()>,
    {
        let mut chunk = Vec::with_capacity(chunk_size);

        while let Some(item) = self.next().await {
            chunk.push(item);

            if chunk.len() >= chunk_size {
                chunk_fn(&chunk)?;
                chunk.clear();
            }
        }

        // Process any remaining items in the final chunk.
        if !chunk.is_empty() {
            chunk_fn(&chunk)?;
        }

        Ok(())
    }

    /// Skip `n` items and return the paginator.
    pub async fn skip(mut self, n: usize) -> Self {
        for _ in 0..n {
            if self.next().await.is_none() {
                break;
            }
        }
        self
    }

    /// Get the current pagination options state.
    pub fn current_pagination(&self) -> &HttpPaginationOptions {
        &self.pagination
    }

    /// Check if there are potentially more items to fetch.
    pub fn has_more(&self) -> bool {
        self.has_more || self.current_index < self.current_batch.len()
    }
}
