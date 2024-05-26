FROM rust:1-bullseye as server-builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

ARG DEBIAN_FRONTEND=noninteractive 

WORKDIR /app

COPY --from=server-builder /app/target/release/furss /app/signet

CMD ["/app/signet"]