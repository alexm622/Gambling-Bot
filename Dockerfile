
#Use the official Rust image as the build environment
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock (to cache dependencies)
COPY Cargo.toml Cargo.lock ./

# Build the dependencies first to cache them
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/gambling-bot*

# Copy the actual source code into the container
COPY . .

# Build the final application
RUN cargo build --release

# Use a smaller base image to reduce size
FROM debian:bullseye-slim

# Install necessary dependencies
RUN apt-get update && apt-get install -y libssl-dev

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/gambling-bot /usr/local/bin/gambling-bot

# Set the entry point of the container to the binary
CMD ["gambling-bot"]
