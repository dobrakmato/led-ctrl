FROM rust:latest AS builder

RUN rustup target add aarch64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev gcc-aarch64-linux-gnu
RUN update-ca-certificates

# Create appuser
ENV USER=led_ctrl
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /led_ctrl

COPY ./ .

RUN cargo build --target aarch64-unknown-linux-musl --release

FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /led_ctrl

# Copy our build
COPY --from=builder /led_ctrl/target/aarch64-unknown-linux-musl/release/led_ctrl ./
ENV PATH="${PATH}:/led_ctrl"

# Use an unprivileged user.
USER led_ctrl:led_ctrl

CMD ["/led_ctrl/led_ctrl"]