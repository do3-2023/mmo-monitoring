FROM debian:bullseye-slim

RUN adduser --disabled-password --gecos '' frontend

WORKDIR /app

COPY --chown=frontend:frontend /target/x86_64-unknown-linux-musl/release/frontend /app/frontend

USER frontend

CMD [ "/app/frontend" ]
