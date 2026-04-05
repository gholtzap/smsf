# Audit Findings (2026-04)

Full endpoint-by-endpoint audit completed across the SMSF codebase. Key fixes applied:

## NF Client Layer

### NrfClient (`src/nf_client/nrf.rs`)
- Fixed: TLS errors now propagated with `.context()` instead of silently ignored
- Fixed: NRF discovery returns empty SearchResult on 404 instead of erroring
- Fixed: Heartbeat auto-reregisters when NRF returns 404 (registration lost)

### AmfClient (`src/nf_client/amf.rs`)
- Fixed: TLS error propagation — `build_client` returns Result, caller handles it
- Fixed: `discover_amf` selects AMF by (lowest priority, highest capacity, lowest load) using `min_by_key` with `Reverse(capacity)`
- Fixed: Falls back to `ipv4_addresses` when no `namf-comm` service with `api_prefix` found
- Fixed: Only selects services with `nf_service_status == Registered`

### OAuth2 Middleware (`src/sbi/middleware/oauth2.rs`)
- Fixed: Returns ProblemDetails JSON (not plain text) for 401/403 errors per 3GPP conventions
- Fixed: Scope validation splits on whitespace (space-separated scopes per OAuth2 spec)
- Fixed: Inserts validated `TokenClaims` into request extensions for downstream handlers

## Router (`src/sbi/server.rs`)
- Added: Fallback handler returns 404 ProblemDetails JSON instead of Axum default
- Added: `DefaultBodyLimit::max(1024 * 1024)` to prevent oversized payloads
- Added: `TraceLayer` for HTTP request logging

## Handler Endpoints
- All six endpoints audited for correct status codes, error handling, input validation
- Each handler uses a scoped AppState containing only its required dependencies
