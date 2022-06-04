FROM docker.io/library/rust:1.60-alpine as builder
WORKDIR /app/
RUN apk update && apk add --no-cache musl-dev
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN cargo build --release

FROM docker.io/library/alpine:latest as runtime
COPY --from=builder /app/target/release/blog_server /usr/local/bin/blog_server
ENTRYPOINT ["/usr/local/bin/blog_server"]
CMD ["/etc/blog/config.toml"]
