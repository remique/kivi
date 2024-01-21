FROM rust:1.75 as builder

RUN USER=root cargo new --bin kivi
WORKDIR /kivi

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/kivi*
RUN cargo build --release --bin client

FROM rust:1.75

COPY --from=builder /kivi/target/release/client .

CMD ["./client"]
