FROM caddy:2.8-builder AS builder
RUN caddy build-modules --modules github.com/caddy-dns/route53

FROM caddy:2.8
COPY --from=builder /usr/bin/caddy /usr/bin/caddy
COPY opt/relay/Caddyfile /etc/caddy/Caddyfile
