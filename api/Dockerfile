FROM rust:1.90.0-alpine3.22 AS build
WORKDIR /usr/src/lucos_configy_api

RUN apk add --no-cache musl-dev pkgconfig openssl-dev

# Leverage Docker caching by only building dependencies before copying in real source
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release || true

COPY src ./src
RUN cargo build --release

FROM alpine:3.22

COPY --from=build /usr/src/lucos_configy_api/target/release/lucos_configy_api /usr/local/bin/lucos_configy_api

CMD ["lucos_configy_api"]
