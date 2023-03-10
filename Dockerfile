FROM --platform=x86_64 rust:1-slim AS build

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev

WORKDIR /home

COPY Cargo.toml .
COPY Cargo.lock .
COPY src src

RUN  --mount=type=cache,target=/home/.cargo cargo build --target x86_64-unknown-linux-musl --release

FROM scratch

COPY --from=build /home/target/x86_64-unknown-linux-musl/release/rust /app

CMD ["/app"]