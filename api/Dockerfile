FROM rust:1.91.1-alpine3.22 AS build
WORKDIR /usr/src/lucos_configy_api

RUN apk add --no-cache musl-dev pkgconfig

COPY .cargo ./.cargo
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM alpine:3.23

COPY --from=build /usr/src/lucos_configy_api/target/release/lucos_configy_api /usr/local/bin/lucos_configy_api
COPY config.yaml .

CMD ["lucos_configy_api"]
