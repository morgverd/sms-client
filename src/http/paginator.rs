//! HTTP request paginator, supporting lazy traversal across large sets

use crate::http::types::HttpPaginationOptions;
use crate::http::error::*;

/// Call a function with an update HttpPaginationOptions for each batch request,
/// simplifying lazy access to large response sets such as messages etc.
pub struct HttpPaginator<T, F, Fut> {
    http_fn: F,
    pagination: HttpPaginationOptions,
    current_batch: Vec<T>,
    current_index: usize,
    has_more: bool,
    initial_limit: u64,
    _phantom: std::marker::PhantomData<Fut>
}
impl<T, F, Fut> HttpPaginator<T, F, Fut>
where
    F: Fn(Option<HttpPaginationOptions>) -> Fut,
    Fut: Future<Output = HttpResult<Vec<T>>>
{

    /// Create the paginator with the http batch generator.
    pub fn new(http_fn: F, pagination: HttpPaginationOptions) -> Self {
        let initial_limit = pagination.limit.unwrap_or(50);

        Self {
            http_fn,
            pagination,
            current_batch: Vec::new(),
            current_index: 0,
            has_more: true,
            initial_limit,
            _phantom: std::marker::PhantomData
        }
    }

    /// Create a paginator with default pagination settings.
    pub fn with_defaults(http_fn: F) -> Self {
        Self::new(
            http_fn,
            HttpPaginationOptions::default()
                .with_limit(50)
                .with_offset(0)
        )
    }

    /// Fetch the next batch of items from the API.
    async fn fetch_next_batch(&mut self) -> HttpResult<bool> {
        log::trace!("Fetching next batch: {:?}", self.pagination);
        let response = (self.http_fn)(Some(self.pagination.clone())).await?;

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
    pub async fn next(&mut self) -> Option<T> {
        if self.current_index >= self.current_batch.len() {

            // If there aren't any-more, then there is nothing to fetch next.
            if !self.has_more {
                return None;
            }

            match self.fetch_next_batch().await {
                Ok(true) => {}, // Successfully fetched more data
                Ok(false) | Err(_) => return None // No more data or error
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
    pub async fn for_each_chuck<C>(mut self, chunk_size: usize, mut chunk_fn: C) -> HttpResult<()>
    where
        C: FnMut(&[T]) -> HttpResult<()>
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