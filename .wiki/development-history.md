# Development History

Chronological summary of SMSF build-out phases.

## Phase 1: Core SMS Protocol Stack
- Basic SMSF Rust service scaffold (Axum + MongoDB)
- SMS encoding (GSM 7-bit, UCS-2), TP-PDU encoding/decoding
- Store-and-forward with retry
- Concatenated SMS and UDH support
- Delivery report endpoint

## Phase 2: NF Integration
- UDM integration for subscriber data lookups
- AMF discovery via NRF + UE reachability checks
- CP-DATA message handling (CP-layer for NAS SMS transport)
- TP-Validity-Period enforcement
- Enhanced delivery status tracking

## Phase 3: Security & SBI Compliance
- MSISDN routing and E.164 normalization
- UDM service discovery via NRF (dynamic, not hardcoded)
- TLS and mTLS support for all SBI interfaces (server + client)
- OAuth 2.0 JWT authentication middleware

## Phase 4: Audit & Hardening (current)
- Full endpoint-by-endpoint audit of all 6 API handlers
- OAuth2 middleware hardened (ProblemDetails responses, scope validation)
- Router hardened (fallback handler, body limit, trace layer)
- NrfClient hardened (TLS propagation, discovery error handling, heartbeat re-registration)
- AmfClient hardened (TLS propagation, AMF selection algorithm, service status filtering)
