FROM rust:latest as build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:latest
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/rust-api /usr/local/bin/rust-api
ENTRYPOINT ["rust-api"]

LABEL org.label-schema.name="rust-api"