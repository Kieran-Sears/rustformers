FROM rust:1.76 AS builder

WORKDIR /app

COPY . .
# RUN apt-get update && apt-get install -y libpq-dev # might solve an issue with lib missing
RUN cargo build --release

FROM debian:bookworm

WORKDIR /app

COPY --from=builder /app/config.json /app/
COPY --from=builder /app/models/ /app/models/
COPY --from=builder /app/target/release/rustformers /app/

CMD ["./rustformers"]
