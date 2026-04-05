## HTTP Endpoints

- [x] `PUT /nsmsf-sms/v1/ue-contexts/:supi` — Activate SMS service for a UE (src/sbi/handlers/activation.rs:18)
- [x] `PATCH /nsmsf-sms/v1/ue-contexts/:supi` — Update UE SMS context via JSON Patch (src/sbi/handlers/update.rs:17)
- [x] `DELETE /nsmsf-sms/v1/ue-contexts/:supi` — Deactivate SMS service for a UE (src/sbi/handlers/deactivation.rs:16)
- [x] `POST /nsmsf-sms/v1/ue-contexts/:supi/sendsms` — Receive uplink (MO) SMS from AMF (src/sbi/handlers/send_sms.rs:24)
- [x] `POST /nsmsf-sms/v1/ue-contexts/:supi/send-mt-sms` — Send downlink (MT) SMS to UE (src/sbi/handlers/send_mt_sms.rs:25)
- [x] `POST /nsmsf-sms/v1/ue-contexts/:supi/delivery-report` — Receive SMS delivery report (src/sbi/handlers/delivery_report.rs:16)
- [x] `GET /health` — Health check endpoint (src/sbi/server.rs:100)

## Middleware

- [x] `oauth2_auth` — OAuth 2.0 JWT authentication middleware (src/sbi/middleware/oauth2.rs:28)

## Router & App State

- [x] `create_router` — Builds the Axum router with all routes and middleware (src/sbi/server.rs:28)

## NF Client Operations — NRF

- [x] `NrfClient::new` — Create NRF client with optional TLS (src/nf_client/nrf.rs:138)
- [x] `NrfClient::build_smsf_profile` — Build NF profile for SMSF registration (src/nf_client/nrf.rs:165)
- [x] `NrfClient::register` — Register SMSF with NRF (src/nf_client/nrf.rs:208)
- [x] `NrfClient::deregister` — Deregister SMSF from NRF (src/nf_client/nrf.rs:246)
- [x] `NrfClient::heartbeat` — Send heartbeat to NRF (src/nf_client/nrf.rs:278)
- [x] `NrfClient::discover` — Discover NF instances via NRF (src/nf_client/nrf.rs:307)
- [x] `NrfClient::start_heartbeat_task` — Background heartbeat with auto re-registration (src/nf_client/nrf.rs:362)

## NF Client Operations — AMF

- [x] `AmfClient::new` / `AmfClient::with_nrf` — Create AMF client (src/nf_client/amf.rs:67)
- [ ] `AmfClient::discover_amf` — Discover AMF via NRF (src/nf_client/amf.rs:102)
- [ ] `AmfClient::check_ue_reachability` — Check UE CM state at AMF (src/nf_client/amf.rs:154)
- [ ] `AmfClient::send_n1n2_message` — Send N1N2 message transfer to AMF (src/nf_client/amf.rs:192)

## NF Client Operations — UDM

- [ ] `UdmClient::new` / `UdmClient::with_nrf` — Create UDM client (src/nf_client/udm.rs:56)
- [ ] `UdmClient::get_udm_uri` — Resolve UDM URI via NRF discovery or fallback (src/nf_client/udm.rs:95)
- [ ] `UdmClient::discover_udm` — Discover UDM via NRF with priority selection (src/nf_client/udm.rs:127)
- [ ] `UdmClient::get_am_data` — Fetch Access and Mobility subscription data (src/nf_client/udm.rs:174)
- [ ] `UdmClient::get_sms_data` — Fetch SMS subscription data (src/nf_client/udm.rs:210)
- [ ] `UdmClient::get_sms_mng_data` — Fetch SMS management subscription data (src/nf_client/udm.rs:246)
- [ ] `UdmClient::get_sms_authorization` — Aggregate SMS authorization from UDM (src/nf_client/udm.rs:288)

## Database Operations

- [ ] `Database::new` — Connect to MongoDB (src/db/mod.rs:15)
- [ ] `Database::save_ue_context` — Insert UE SMS context (src/db/mod.rs:29)
- [ ] `Database::update_ue_context` — Replace UE SMS context by SUPI (src/db/mod.rs:37)
- [ ] `Database::delete_ue_context` — Delete UE SMS context (src/db/mod.rs:45)
- [ ] `Database::get_ue_context` — Find UE SMS context by SUPI (src/db/mod.rs:53)
- [ ] `Database::save_sms_record` — Insert SMS record (src/db/mod.rs:60)
- [ ] `Database::update_sms_status` — Update SMS delivery status (src/db/mod.rs:68)
- [ ] `Database::update_sms_status_with_reason` — Update SMS status with failure reason (src/db/mod.rs:82)
- [ ] `Database::get_sms_record` — Find SMS record by ID (src/db/mod.rs:97)
- [ ] `Database::get_pending_retries` — Query retryable SMS records (src/db/mod.rs:104)
- [ ] `Database::increment_retry_count` — Increment retry count and set next retry time (src/db/mod.rs:130)
- [ ] `Database::mark_expired` — Mark SMS record as expired (src/db/mod.rs:151)
- [ ] `Database::load_all_ue_contexts` — Load all UE contexts on startup (src/db/mod.rs:159)

## Context Store

- [ ] `UeSmsContextStore` — In-memory DashMap store for UE SMS contexts (src/context/ue_sms_context.rs:64)
- [ ] `UeSmsContext::from_data` / `update_from_data` / `to_data` — Context data conversion (src/context/ue_sms_context.rs:23)

## SMS Delivery

- [ ] `SmsDeliveryService::deliver_mt_sms` — MT-SMS delivery orchestration (src/sms/delivery.rs:26)
- [ ] `SmsDeliveryService::attempt_delivery` — Attempt delivery via AMF (src/sms/delivery.rs:91)
- [ ] `SmsDeliveryService::classify_delivery_error` — Classify error into delivery status (src/sms/delivery.rs:110)

## SMS Retry

- [ ] `SmsRetryService::start` — Background retry loop (src/sms/retry.rs:25)
- [ ] `SmsRetryService::process_retries` — Process pending retries (src/sms/retry.rs:39)
- [ ] `SmsRetryService::calculate_backoff` — Exponential backoff calculation (src/sms/retry.rs:115)

## SMS Status Reports

- [ ] `StatusReportService::send_status_report` — Generate and send TP-STATUS-REPORT (src/sms/status_report.rs:22)
- [ ] `StatusReportService::handle_delivery_status_change` — Trigger status report on final status (src/sms/status_report.rs:112)

## TPDU Encoding/Decoding

- [ ] `TpSubmit::new` / `encode` / `decode` — TP-SUBMIT PDU (src/sms/tpdu.rs:122)
- [ ] `TpDeliver::new` / `encode` / `decode` — TP-DELIVER PDU (src/sms/tpdu.rs:273)
- [ ] `TpStatusReport::new` / `encode` / `decode` — TP-STATUS-REPORT PDU (src/sms/tpdu.rs:53)
- [ ] `encode_address` / `decode_address` — TPDU address encoding (src/sms/tpdu.rs:404)
- [ ] `encode_timestamp` / `decode_timestamp` — SCTS timestamp encoding (src/sms/tpdu.rs:472)
- [ ] `encode_validity_period` / `decode_validity_period` — VP relative format (src/sms/tpdu.rs:519)

## CP Layer

- [ ] `CpMessage::encode` / `decode` — CP-DATA, CP-ACK, CP-ERROR encoding (src/sms/cp.rs:54)

## RP Layer

- [ ] `RpMessage::encode` / `decode` — RP-DATA, RP-ACK, RP-ERROR, RP-SMMA encoding (src/sms/rp.rs:180)

## Text Encoding

- [ ] `encode_gsm7` / `decode_gsm7` — GSM 7-bit encoding/decoding (src/sms/encoding.rs:54)
- [ ] `encode_ucs2` / `decode_ucs2` — UCS-2 encoding/decoding (src/sms/encoding.rs:144)
- [ ] `auto_detect_encoding` — Auto-detect GSM-7 vs UCS-2 (src/sms/encoding.rs:164)
- [ ] `encode_text` / `decode_text` — Dispatch to correct encoder (src/sms/encoding.rs:173)

## UDH (User Data Header)

- [ ] `UserDataHeader::parse` / `encode` — UDH parsing and encoding (src/sms/udh.rs:44)
- [ ] `InformationElement::encode` — IE encoding for concat, port, unknown (src/sms/udh.rs:208)

## Concatenation

- [ ] `split_long_message_submit` — Split long SMS into concatenated TP-SUBMITs (src/sms/concatenation.rs:21)
- [ ] `split_long_message_deliver` — Split long SMS into concatenated TP-DELIVERs (src/sms/concatenation.rs:92)

## Reassembly

- [ ] `MessageReassembly::add_part_submit` — Reassemble concatenated MO-SMS (src/sms/reassembly.rs:42)
- [ ] `MessageReassembly::add_part_deliver` — Reassemble concatenated MT-SMS (src/sms/reassembly.rs:91)
- [ ] `MessageReassembly::reassemble_message` — Combine parts into full text (src/sms/reassembly.rs:140)
- [ ] `MessageReassembly::cleanup_expired` — Remove timed-out partial messages (src/sms/reassembly.rs:165)

## Routing

- [ ] `E164Number::parse` — Parse and validate E.164 numbers (src/sms/routing.rs:305)
- [ ] `Msisdn::parse` — Parse MSISDN with E.164 validation (src/sms/routing.rs:351)
- [ ] `ServiceCentreAddress::new` / `encode` / `decode` — SCA handling (src/sms/routing.rs:386)
- [ ] `SmsRouter::normalize_number` — Normalize national/international numbers (src/sms/routing.rs:502)
- [ ] `SmsRouter::route_sms` — Route SMS with domestic/international classification (src/sms/routing.rs:531)
- [ ] `encode_ton_npi` / `decode_ton_npi` — Type-of-Number / NPI encoding (src/sms/routing.rs:76)

## Multipart Parsing

- [ ] `parse_multipart_sms` — Parse multipart/related SMS request body (src/sbi/multipart.rs:5)

## TLS Configuration

- [ ] `load_tls_config` — Load server TLS/mTLS configuration (src/tls.rs:13)
- [ ] `build_client_config` — Build rustls client config for outbound TLS (src/tls.rs:78)

## Configuration

- [ ] `Config::from_file` — Load config from JSON file with env overrides (src/config.rs:87)
- [ ] `Config::from_env` — Build config entirely from environment variables (src/config.rs:94)
- [ ] `Config::apply_env_overrides` — Apply env var overrides to file-based config (src/config.rs:186)

## Initialization & Lifecycle

- [ ] `main` — Application startup, service wiring, graceful shutdown (src/main.rs:26)
- [ ] `shutdown_signal` — SIGTERM/Ctrl-C signal handler (src/main.rs:143)

## Models & Types

- [ ] `ProblemDetails` — 3GPP problem details response model (src/sbi/models.rs:83)
- [ ] `UeSmsContextData` — UE SMS context request/response model (src/sbi/models.rs:46)
- [ ] `SmsRecord` — SMS record persistence model (src/sms/types.rs:17)
- [ ] `SmsDeliveryStatus` — Delivery status enum (src/sms/types.rs:4)
- [ ] `NFProfile` / `NFService` — NRF NF profile models (src/nf_client/nrf.rs:62)
