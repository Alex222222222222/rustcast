FROM rust:latest AS builder

# Clone and build RustCast
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:stable-slim

# Create non-root user
RUN addgroup --system rustcast && adduser --system --ingroup rustcast rustcast

# Create directories for application data
RUN mkdir -p /app/config /app/music /app/logs \
    && chown -R rustcast:rustcast /app

# Copy the built binary from builder
COPY --from=builder /build/target/release/rustcast /usr/local/bin/rustcast
# Give the binary the correct permissions
RUN chown rustcast:rustcast /usr/local/bin/rustcast
# make the binary executable
RUN chmod +x /usr/local/bin/rustcast

# Create a default configuration file
# # add the default configuration file
COPY <<EOF /app/config/config.json
{
    "playlists": {
        "main": {
            "name": "RustCast Default Stream",
            "child": {
                "LocalFolder": {
                    "folder": "/app/music",
                    "repeat": true,
                    "shuffle": true,
                    "fail_over": "Silent"
                }
            }
        }
    },
    "file_provider": {},
    "outputs": [
        {
            "host": "0.0.0.0",
            "port": 8080,
            "path": "/",
            "playlist": "main"
        }
    ],
    "log_level": "info",
    "log_file": ["stdout", "/app/logs/rustcast.log"]
}
EOF
RUN chown rustcast:rustcast /app/config/config.json

# Install runtime dependencies
RUN apt-get update \
    && apt-get install -y libssl3 ca-certificates\
    && apt clean \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Switch to non-root user
USER rustcast

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/rustcast"]

# Run with default configuration
CMD ["/app/config/config.json"]
