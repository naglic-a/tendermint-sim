FROM rust:1.75-slim-bookworm as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/tendermint-sim .
CMD ["./tendermint-sim"]
