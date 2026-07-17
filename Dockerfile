# array-test container image (C1, D36): the evidence-producing environment, pinned.
#
# Multi-stage: a Rust builder compiles the release binary; the runtime stage carries only
# the binary plus the tools the guarantee tiers need — cbmc (the `proved` tier's bounded
# model checker) and python3-hypothesis (the `property` tier) — so both tiers are live by
# default instead of "provision it yourself". No Rust toolchain ships in the runtime image.
#
# Both stages are the same Debian release (trixie) so the builder's glibc matches the
# runtime's. Consumers should pin the *published* image by digest (`@sha256:…`, never
# `:latest`) — that is the same content-addressing discipline the engine applies to the
# tested toolchain (toolchain.lock → toolchain_hash), extended to the runner's environment.
#
# Run modes (C2 — see README "Container image"):
#   plain `docker run`                                → EnvOnly isolation (recorded as such)
#   `--privileged` or `--cap-add=SYS_ADMIN` (+userns) → full sandbox (netns + RO mounts)
# The image is immutable; the *container* additionally needs `--read-only` if you want an
# immutable runtime filesystem.

FROM rust:1-slim-trixie AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends cbmc python3 python3-hypothesis \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/array-test /usr/local/bin/array-test
# The sprint-loops Test-phase shim (T14) and the committed example workspaces, so the
# image is demonstrable (and CI-provable) standalone.
COPY adapters/sprint-loops/array-test-phase.sh /usr/local/bin/array-test-phase
COPY examples /opt/array-test/examples
RUN chmod +x /usr/local/bin/array-test-phase

ENTRYPOINT ["array-test"]
CMD ["help"]
