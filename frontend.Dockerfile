FROM debian:bullseye-slim

RUN apt install -y libssl-dev

WORKDIR /app

COPY /target/x86_64-unknown-linux-musl/release/frontend /app/frontend

CMD [ "/app/frontend" ]
