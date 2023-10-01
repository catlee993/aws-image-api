FROM rust:latest as builder

WORKDIR /usr/src/rust-api
COPY ./ ./

RUN cargo install --path .

FROM rust:latest

COPY --from=builder /usr/local/cargo/bin/rust-api /usr/local/bin/rust-api

CMD ["rust-api"]