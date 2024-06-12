[private]
default:
  @just --list --unsorted

run:
  cargo run

compile DATABASE_URL = "postgresql://postgres:postgres@localhost:5432/person":
  #!/usr/bin/env bash
  cargo sqlx prepare --workspace
  sqlx migrate run --database-url {{DATABASE_URL}} --source=person/migrations
  docker run --rm \
    -v cargo-cache:/root/.cargo \
    -v $PWD:/volume \
    --network host \
    -e DATABASE_URL={{DATABASE_URL}} \
    -w /volume \
    -t clux/muslrust \
    cargo build --release

docker-build-all TAG: compile
  docker build -t ghcr.io/do3-2023/mmo-monitoring/frontend:{{TAG}} -f frontend.Dockerfile . --push
  docker build -t ghcr.io/do3-2023/mmo-monitoring/person:{{TAG}} -f person.Dockerfile . --push
