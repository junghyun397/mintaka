# mintaka-server
rest api provider for mintaka-webui and GomokuBot

## Build with mintaka-webui
```shell
(cd mintaka_webui && pnpm run requirements)
(cd mintaka_webui && pnpm run build)
cargo run -p mintaka_server -- --webui
```

## Options

Use `--help` to list options. Pass option values as separate arguments, for example `--address 127.0.0.1:8085 -c 4 -m 4096`.

CLI options override environment variables. Supported variables are `WEBUI`, `ADDRESS`, `CORES`, `MEMORY_LIMIT_MIB`, `TLS_CERT`, `TLS_KEY`, `TLS_RENEW`, and `API_PASSWORD`. Boolean variables accept `true` or `false`.

## TLS encryption

Both the certificate and key are required, whether supplied through CLI options or environment variables. The default listen address is `127.0.0.1:8445` with TLS and `127.0.0.1:8085` otherwise.

```shell
cargo run -p mintaka_server -- \
  --tls-cert /etc/letsencrypt/live/example.com/cert.pem \
  --tls-key /etc/letsencrypt/live/example.com/privkey.pem \
  --tls-renew
```
