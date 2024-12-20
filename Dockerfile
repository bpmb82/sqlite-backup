FROM rust:1.82-alpine AS builder

WORKDIR /build

RUN apk add musl-dev sqlite-dev sqlite-libs

COPY src ./src
COPY Cargo.toml .

RUN arch=$(arch) && \
    cargo build --release --target $arch-unknown-linux-musl && \
    mv /build/target/$arch-unknown-linux-musl/release/sqlite-backup /build/sqlite-backup

FROM scratch

COPY --from=builder /build/sqlite-backup .

CMD ["/sqlite-backup", "daemon"]