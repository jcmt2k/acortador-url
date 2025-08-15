# Stage 1: Build the application
FROM rust:1.82 as builder

WORKDIR /usr/src/app

# Install dependencies
RUN apt-get update && apt-get install -y libsqlite3-dev

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Stage 2: Create the final image
FROM debian:stable-slim

WORKDIR /usr/src/app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/acortador-url .

# Copy the templates directory
COPY templates ./templates

# Expose the application port
EXPOSE 3000

# Set the environment variables
ENV DATABASE_URL=sqlite:db.sqlite
ENV HOST=0.0.0.0
ENV PORT=3000

# Run the application
CMD ["./acortador-url"]
