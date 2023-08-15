FROM rust:1.65.0

WORKDIR /app

RUN apt update && apt install lld clang -y

COPY . .

ENV DATABASE_URL "postgres://postgres:password@localhost:5432/ventrix"

RUN cargo build --release

ENV SQLX_OFFLINE true
ENV APP_ENVIRONMENT production

ENTRYPOINT ["./target/release/ventrix"]
