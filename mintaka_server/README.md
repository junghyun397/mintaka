# mintaka-server
mintaka rest api 

## With web ui
```shell
(cd mintaka_webui && pnpm run requirements)
(cd mintaka_webui && pnpm run build)
cargo run -p mintaka_server -- --webui
```

## TLS encryption

```shell
./mintaka-server --tls-cert /etc/ssl/certs/cert.pem --tls-key /etc/ssl/certs/key.pem --tls-renew
```
