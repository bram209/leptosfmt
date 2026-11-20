FROM rust:1.81.0-alpine AS build

RUN apk add --no-cache musl-dev~=1.2

COPY . /app
WORKDIR /app
RUN cargo build --package leptosfmt \
                --target x86_64-unknown-linux-musl \
                --locked \
                --release

FROM scratch
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/leptosfmt /leptosfmt
ENTRYPOINT ["/leptosfmt"]
