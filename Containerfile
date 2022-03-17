FROM docker.io/rust:1.58.1-alpine AS builder
WORKDIR /builder

RUN apk add musl-dev

ADD . .
RUN cargo build --release

FROM scratch
COPY --from=builder /builder/target/release/meishu .

CMD ["/meishu"]
