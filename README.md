# SMSF (SMS Function)

5G SMS Function implementation in Rust following 3GPP TS 29.540 specification.

## Features

- SMS Service Activation/Deactivation
- SMS Service Parameter Update (JSON Patch RFC 6902)
- Uplink SMS (UE → Network)
- Downlink SMS (Network → UE)
- NRF Integration (Registration, Discovery, Heartbeat)
- MongoDB Persistence
- ETag Support for Conditional Updates
- Multipart SMS Payload Handling

## API Endpoints

- `PUT /nsmsf-sms/v1/ue-contexts/{supi}` - Activate SMS service
- `PATCH /nsmsf-sms/v1/ue-contexts/{supi}` - Update SMS context
- `DELETE /nsmsf-sms/v1/ue-contexts/{supi}` - Deactivate SMS service
- `POST /nsmsf-sms/v1/ue-contexts/{supi}/sendsms` - Send uplink SMS
- `POST /nsmsf-sms/v1/ue-contexts/{supi}/send-mt-sms` - Send downlink SMS
- `GET /health` - Health check

## Configuration

Environment variables:
- `SBI_BIND_ADDR` - SBI bind address (default: 0.0.0.0)
- `SBI_BIND_PORT` - SBI port (default: 8085)
- `MONGODB_URI` - MongoDB connection string
- `NRF_URI` - NRF base URI
- `NF_INSTANCE_ID` - NF instance ID (auto-generated if not set)
- `SMSF_HOST` - SMSF hostname/IP
- `CONFIG_FILE` - Path to JSON config file (optional)

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run
```

## Docker

```bash
docker build -t smsf .
docker run -p 8085:8085 \
  -e MONGODB_URI=mongodb://mongodb:27017 \
  -e NRF_URI=http://nrf:8081 \
  smsf
```