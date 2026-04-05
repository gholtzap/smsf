# SMSF Architecture

Rust-based SMSF (SMS Function) for a custom 5G core, built with Axum + Tokio.

## Module Layout

- `src/main.rs` — startup, wiring, graceful shutdown
- `src/config.rs` — JSON file or env-var config (Config, TlsConfig, OAuth2Config, RetryConfig)
- `src/sbi/` — HTTP API layer
  - `server.rs` — Axum router, AppState, route definitions
  - `handlers/` — one file per endpoint (activation, deactivation, update, send_sms, send_mt_sms, delivery_report)
  - `middleware/oauth2.rs` — JWT Bearer token validation middleware
  - `models.rs` — SBI request/response types (ProblemDetails, Guami, UserLocation, etc.)
  - `multipart.rs` — multipart/related parsing for N1N2 payloads
- `src/nf_client/` — outbound HTTP clients
  - `nrf.rs` — NRF registration, discovery, heartbeat (with auto-reregistration on 404)
  - `amf.rs` — AMF discovery (via NRF), UE reachability check, N1N2 message transfer
  - `udm.rs` — UDM subscriber data lookups
- `src/sms/` — SMS protocol stack
  - `tpdu.rs`, `rp.rs`, `cp.rs` — 3GPP TP-PDU, RP-layer, CP-layer encoding/decoding
  - `encoding.rs` — GSM 7-bit / UCS-2 character encoding
  - `udh.rs` — User Data Header parsing
  - `concatenation.rs`, `reassembly.rs` — multi-part SMS handling
  - `routing.rs` — MSISDN routing, E.164 normalization
  - `delivery.rs` — SMS delivery orchestration via AMF
  - `retry.rs` — store-and-forward with exponential backoff
  - `status_report.rs` — delivery status report generation
  - `types.rs` — shared SMS types
- `src/tls.rs` — TLS server config (mTLS optional) + client config builder
- `src/db/` — MongoDB persistence (UE contexts, SMS records, IP allocations)
- `src/context/` — in-memory UE SMS context store (DashMap-backed, restored from DB on startup)

## Key Patterns

- Each handler module defines its own `AppState` subset (only the deps it needs)
- NF clients take optional `TlsConfig` and build reqwest clients with rustls when TLS enabled
- NRF heartbeat runs as a background tokio task; auto-reregisters on 404
- OAuth2 middleware is a no-op when `oauth2.enabled = false`
- AMF selection: lowest priority, highest capacity, lowest load (multi-key sort)
- N1N2 messages to AMF use multipart/related with base64 SMS payload
- Config supports both JSON file (`CONFIG_FILE` env) and pure env-var bootstrap

## API Routes

All protected routes require OAuth2 JWT when enabled:

| Method | Path | Handler |
|--------|------|---------|
| PUT | `/nsmsf-sms/v1/ue-contexts/:supi` | activation |
| PATCH | `/nsmsf-sms/v1/ue-contexts/:supi` | update |
| DELETE | `/nsmsf-sms/v1/ue-contexts/:supi` | deactivation |
| POST | `/nsmsf-sms/v1/ue-contexts/:supi/sendsms` | send_sms (uplink) |
| POST | `/nsmsf-sms/v1/ue-contexts/:supi/send-mt-sms` | send_mt_sms (downlink) |
| POST | `/nsmsf-sms/v1/ue-contexts/:supi/delivery-report` | delivery_report |
| GET | `/health` | health check (unprotected) |

## Dependencies

Axum 0.7, reqwest 0.12 (rustls), mongodb 3, rustls 0.23, jsonwebtoken 9, dashmap 6, chrono, multer 3, base64 0.22.

## Inter-NF Communication

- **NRF**: Registration (PUT), deregistration (DELETE), heartbeat (PUT), discovery (GET)
- **AMF**: UE context check (GET), N1N2 message transfer (POST multipart/related)
- **UDM**: Subscriber data retrieval
