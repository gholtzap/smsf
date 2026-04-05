# Known Issues & Gotchas

## AMF N1N2 Message Format
The N1N2 message transfer to AMF uses multipart/related with base64-encoded SMS payload. The AMF must support this format. If delivery fails, check that the AMF's `/namf-comm/v1/ue-contexts/:supi/n1-n2-messages` endpoint accepts `multipart/related` with `application/vnd.3gpp.sms` content type.

## NRF Heartbeat 404 Recovery
If the NRF restarts or loses state, the SMSF heartbeat will get 404. The NrfClient auto-reregisters on 404, but there's a brief window where the SMSF is undiscoverable.

## OAuth2 Middleware Bypass
When `oauth2.enabled = false`, the middleware is a complete passthrough. No token validation occurs. This is intentional for development but should always be enabled in production.

## Config Precedence
When using `CONFIG_FILE`, env vars still override file values via `apply_env_overrides()`. This can cause confusion if both are set.

## MongoDB Connection
The `MONGODB_URI` must not be committed to source. The CLAUDE.md explicitly prohibits pushing MongoDB credentials to GitHub.
