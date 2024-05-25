FROM rust:alpine as builder

WORKDIR /app

COPY Cargo.toml ./
COPY src ./src
COPY .cargo ./.cargo
COPY .rustfmt.toml ./
RUN apk add --no-cache --virtual .build-deps build-base \
 && rustup toolchain install nightly \
 && rustup component add --toolchain nightly rustfmt clippy \
 && cargo +nightly fmt --all -- --check \
 && cargo +nightly clippy --all-targets --all-features -- -D warnings \
 && cargo test \
 && cargo build --release \
 && apk del .build-deps \
 && rm -rf /var/cache/apk/* \
 && cp target/release/semrel /semrel \
 && strip /semrel \
 && rm -rf /app

FROM rust:alpine as runtime
COPY --from=builder /semrel /semrel
ENTRYPOINT ["/semrel"]
