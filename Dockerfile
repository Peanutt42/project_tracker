# Use an official Rust image to build the project
FROM rust:1.83.0 AS builder

# Install dependencies for the build
RUN apk add --no-cache musl-dev build-base

# Set the working directory inside the container
WORKDIR /project_tracker

# Copy source code into the container
COPY . .

# Build the Rust application
RUN cd project_tracker_server && cargo build --release

# Create a smaller runtime image
FROM alpine:latest

# Install any runtime dependencies
RUN apk add --no-cache ca-certificates

# Set the working directory
WORKDIR /project_tracker

# Copy the compiled binary from the builder stage
COPY --from=builder /project_tracker/target/release/project_tracker_server /project_tracker/project_tracker_server

# Create a volume for persistent storage
VOLUME ["/data"]

# Expose the ports
# http
EXPOSE 80
# gui native client ws
EXPOSE 8080

# Set the entrypoint and default arguments
ENTRYPOINT ["/project_tracker/project_tracker_server"]
CMD ["/data"]