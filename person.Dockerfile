FROM debian:bullseye-slim

RUN adduser --disabled-password --gecos '' person

WORKDIR /app

COPY --chown=person:person /target/x86_64-unknown-linux-musl/release/person /app/person

COPY --chown=person:person person/migrations /app/migrations

USER person

CMD [ "/app/person" ]
