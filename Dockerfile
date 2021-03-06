# syntax = docker/dockerfile-upstream:1.2.0

ARG RUST_IMAGE

# Base target.

FROM ${RUST_IMAGE} AS base
RUN apk add --no-cache curl musl-dev protoc
RUN rustup component add rustfmt clippy
WORKDIR /src
COPY Cargo.lock .
COPY Cargo.toml .
RUN mkdir .cargo
RUN cargo vendor > .cargo/config

# Src target

FROM base AS src
COPY hack hack
COPY src src
COPY tests tests
COPY build.rs build.rs
ARG PROTO_FILE
ADD ${PROTO_FILE} ./proto/

# Lint target.

FROM src AS lint
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/usr/local/target \
  cargo clippy --all-targets --all-features -- -D warnings

# Test target.

FROM src AS test
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/usr/local/target \
  cargo test --frozen --locked --offline --target-dir=/usr/local/target --release --workspace --no-fail-fast -- --nocapture

# Build target.

FROM src AS build
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/usr/local/target \
  cargo build --frozen --locked --offline --target-dir=/usr/local/target --release --bins
ARG TARGETPLATFORM
ENV TARGETPLATFORM=${TARGETPLATFORM}
RUN --mount=type=cache,target=/usr/local/target \
  ./hack/binaries.sh

# Artifacts target.

FROM scratch AS artifacts
COPY --from=build /binaries /binaries
COPY --from=build /src/src/spec /src/spec
COPY --from=build /src/proto /proto

# Image target.

FROM scratch AS image
COPY --from=artifacts /binaries/engine /
COPY --from=artifacts /binaries/runtime /
COPY --from=artifacts /binaries/plugins /system/plugins
ENTRYPOINT [ "/engine" ]
