FROM alpine:latest

COPY ./target/armv7-unknown-linux-musleabihf/release/hue-notify .

CMD ["./hue-notify"]
