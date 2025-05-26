FROM rust:1.80.1 AS builder
RUN apt update && apt install -y clang 
WORKDIR /build
COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock ./Cargo.lock
COPY build.rs ./build.rs
COPY src/ ./src/
COPY lib/ ./lib/
RUN cargo build --release

FROM debian:12.6-slim AS runner
WORKDIR /app
COPY --from=builder /build/target/release/cma-server ./
COPY config.yaml ./config.yaml
COPY cma-recorder.yaml ./cma-recorder.yaml
COPY lib/ ./lib/
CMD ./cma-server -- --config config.yaml 
#cma-recorder.yaml