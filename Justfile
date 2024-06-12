[private]
default:
  @just --list --unsorted

run:
  cargo run

compile:
  #!/usr/bin/env bash
  docker run --rm \
    -v cargo-cache:/root/.cargo \
    -v $PWD:/volume \
    -e SQLX_OFFLINE=true \
    -w /volume \
    -t clux/muslrust \
    cargo build --release

docker-build-all TAG: compile
  docker build -t ghcr.io/do3-2023/mmo-monitoring/frontend:{{TAG}} -f frontend.Dockerfile . --push
  docker build -t ghcr.io/do3-2023/mmo-monitoring/person:{{TAG}} -f person.Dockerfile . --push
