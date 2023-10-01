FROM arm64v8/rust:latest as builder

WORKDIR /usr/src/rust-api
COPY ./ ./

RUN cargo install --path .

FROM arm64v8/rust:latest

COPY --from=builder /usr/local/cargo/bin/rust-api /usr/local/bin/rust-api

CMD ["rust-api"]