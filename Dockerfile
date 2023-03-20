FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=beacon-api-checker
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /beacon-api-checker

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /beacon-api-checker

# Copy our build
COPY --from=builder /beacon-api-checker/target/x86_64-unknown-linux-musl/release/beacon-api-checker ./

# Use an unprivileged user.
USER beacon-api-checker:beacon-api-checker

ENTRYPOINT ["/beacon-api-checker/beacon-api-checker"]
