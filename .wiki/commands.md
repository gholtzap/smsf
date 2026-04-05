# Useful Commands

## Build
```
cd /Users/gmh/dev/telco/smsf && cargo build
```

## Build in Docker (from telco root)
```
cd /Users/gmh/dev/telco && docker-compose build --no-cache smsf
```

## Run full stack
```
cd /Users/gmh/dev/telco && ./start.sh
```

## Check logs
```
docker-compose logs -f smsf
```

## Environment Variables
- `CONFIG_FILE` — path to JSON config (alternative to env vars)
- `SBI_BIND_ADDR` — bind address (default: 0.0.0.0)
- `SBI_BIND_PORT` — bind port (default: 8085)
- `MONGODB_URI` — MongoDB connection string
- `NRF_URI` — NRF base URL (default: http://127.0.0.1:8081)
- `UDM_URI` — UDM base URL (default: http://127.0.0.1:8083)
- `SMSF_HOST` — advertised hostname for NRF registration
- `NF_INSTANCE_ID` — UUID for NRF identity (auto-generated if not set)
- `TLS_ENABLED`, `TLS_CERT_PATH`, `TLS_KEY_PATH` — server TLS
- `TLS_CLIENT_CERT_PATH`, `TLS_CLIENT_KEY_PATH`, `TLS_CLIENT_CA_PATH` — mTLS
- `TLS_REQUIRE_CLIENT_CERT` — enforce mTLS
- `OAUTH2_ENABLED`, `OAUTH2_ISSUER`, `OAUTH2_AUDIENCE`, `OAUTH2_REQUIRED_SCOPE`, `JWT_SECRET` — OAuth2
