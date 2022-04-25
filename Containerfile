FROM docker.io/rust:1.58.1-alpine AS builder
WORKDIR /builder

RUN apk add musl-dev npm

ADD . .
RUN npx tailwindcss -c tailwind.config.js -i tailwind.css -o resources/style.css
RUN cargo build --release

FROM scratch
COPY --from=builder /builder/target/release/meishu .
COPY --from=builder /builder/resources ./resources

CMD ["/meishu"]
