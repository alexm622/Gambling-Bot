
ARG BUILD_MODE=release

# Use the official Rust image as the build environment
FROM rust:trixie AS builder

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

# Build the final application in the requested mode
RUN if [ "$BUILD_MODE" = "release" ]; then \
        cargo build --release; \
    else \
        cargo build; \
    fi

# Use a smaller base image to reduce size
FROM debian:trixie-slim
ARG BUILD_MODE=release

# Install necessary dependencies
RUN apt-get update && apt-get install -y libssl-dev

# Set the working directory for runtime so secrets.csv can be read from ./secrets.csv
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/${BUILD_MODE}/gambling-bot /app/gambling-bot

# Copy secrets file into the runtime image
COPY secrets.csv ./

# Enable developer logging by default
ENV RUST_LOG=gambling_bot=trace

# Set the entry point of the container to the binary
CMD ["./gambling-bot"]
