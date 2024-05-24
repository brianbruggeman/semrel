FROM rust:alpine as builder

WORKDIR /app

COPY Cargo.toml ./
COPY src ./src

# openssl is required due to git2.  Gitoxide supports rustls, but the api and implementation is atrocious.
RUN apk add --no-cache build-base && \
    cargo build --release && \
    cargo test

FROM scratch as runtime
COPY --from=builder /app/target/release/semrel /semrel
CMD ["/semrel"]
