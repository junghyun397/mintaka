# mintaka-server
rest api provider for mintaka-webui and GomokuBot

## Build with mintaka-webui
```shell
(cd mintaka_webui && pnpm run requirements)
(cd mintaka_webui && pnpm run build)
cargo run -p mintaka_server -- --webui
```

## TLS encryption
```shell
./mintaka-server \
  --tls-cert /etc/letsencrypt/live/example.com/cert.pem \
  --tls-key /etc/letsencrypt/live/example.com/privkey.pem \
  --tls-renew
```

## Configuration
* `mintaka-server --help` to see all options.
* Set `max_config.toml` to apply max config.
