# Based on https://kerkour.com/rust-small-docker-image
FROM rustlang/rust:nightly AS builder

WORKDIR /server/ruff

# Create appuser
ENV USER=server
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

COPY ./ /server/ruff

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /server

# Copy our build
COPY --from=builder /server/ruff/target/x86_64-unknown-linux-musl/release/ruff ./

# Use an unprivileged user.
USER server:server

CMD ["/server/ruff"]
