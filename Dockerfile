FROM rust:1-bullseye as builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

ARG DEBIAN_FRONTEND=noninteractive 

WORKDIR /app

COPY --from=builder /app/target/release/furss /app/furss

CMD ["/app/furss"]