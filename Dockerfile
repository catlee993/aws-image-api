# Use a Rust image based on Alpine Linux with musl libc
FROM rust:latest as build

COPY ./ ./

# Build your program for release
RUN cargo build --release

# Run the binary
CMD ["./target/release/rust-api"]