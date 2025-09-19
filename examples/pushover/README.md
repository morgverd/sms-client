# Pushover SMS

An example usage of `sms-client` (websocket only) that sends incoming SMS messages to Pushover for device notifications!

## Env vars

| Key                    | Example                   | Description                                       | Required |
|------------------------|---------------------------|---------------------------------------------------|----------|
| `SMS_PUSHOVER_WS_URL`  | `wss://localhost:3000/ws` | SMS-API websocket events connection URL.          | Yes      |
| `SMS_PUSHOVER_WS_CERT` | `/home/path/cert.crt`     | A certificate file to use for secure connections. | No       |
| `SMS_PUSHOVER_WS_AUTH` | `test`                    | SMS-API websocket authorization.                  | No       |
| `SMS_PUSHOVER_TOKEN`   | `xxxxxxxxxxxxx`           | Pushover app sender key.                          | Yes      |
| `SMS_PUSHOVER_USERS`   | `abc,def,ghi`             | A set of Pushover user keys, comma seperated.     | Yes      |
