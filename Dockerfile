# --- Build Stage ---
FROM rust:1.65.0 AS build

WORKDIR /app

# Install necessary dependencies for building
RUN apt update && apt install lld clang -y

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

# --- Runtime Stage ---
FROM debian:buster

# You might not need to install libssl1.1 manually now, but if the error persists, uncomment the line below:
 RUN apt-get update && apt-get install -y libssl1.1 && rm -rf /var/lib/apt/lists/*

# Set environment variables
ENV DATABASE_URL="postgres://postgres:password@localhost:5432/ventrix" \
    SQLX_OFFLINE=true \
    APP_ENVIRONMENT=production

# Create a directory for the application
WORKDIR /app

# Copy only the built application from the build stage
COPY --from=build /app/target/release/ventrix /app
COPY --from=build /app/configuration /app/configuration


# Set the entry point for the Docker container
ENTRYPOINT ["/app/ventrix"]

