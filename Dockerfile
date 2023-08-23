# --- Build Stage ---
FROM rust:1.65.0 AS build

RUN USER=root cargo new --bin app
RUN touch /app/src/lib.rs
WORKDIR /app

# Install necessary dependencies for building
RUN apt update && apt install lld clang -y

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src
COPY ./migrations ./migrations
COPY ./configuration ./configuration

# build for release
RUN rm -R ./target/
RUN cargo build --release

# Build the application
RUN cargo build --release

# --- Runtime Stage ---
FROM debian:buster

# You might not need to install libssl1.1 manually now, but if the error persists, uncomment the line below:
RUN apt-get update && apt-get install -y libssl1.1 && rm -rf /var/lib/apt/lists/*

# Create a directory for the application
WORKDIR /app

# Copy only the built application from the build stage
COPY --from=build /app/target/release/ventrix /app
COPY --from=build /app/configuration /app/configuration

# Set the entry point for the Docker container
ENTRYPOINT ["/app/ventrix"]

