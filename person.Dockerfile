FROM debian:bullseye-slim

WORKDIR /app

COPY /target/x86_64-unknown-linux-musl/release/person /app/person

CMD [ "/app/person" ]
