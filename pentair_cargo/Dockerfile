# syntax=docker/dockerfile:1.4
FROM rust:buster AS base

ENV USER=root
ENV PENTAIR_ADDRESS=0.0.0.0
ENV PENTAIR_ENV=development

WORKDIR /code
RUN cargo init
COPY Cargo.toml /code/Cargo.toml
RUN cargo fetch
COPY . /code

FROM base AS development

EXPOSE 3000

CMD [ "cargo", "run", "--offline" ]

FROM base AS dev-envs

EXPOSE 3000
RUN <<EOF
apt-get update
apt-get install -y --no-install-recommends git
EOF

RUN <<EOF
useradd -s /bin/bash -m vscode
groupadd docker
usermod -aG docker vscode
EOF
# install Docker tools (cli, buildx, compose)
COPY --from=gloursdocker/docker / /
CMD [ "cargo", "run", "--offline" ]

FROM base AS builder

RUN cargo build --release --offline

FROM debian:buster-slim

ENV PENTAIR_ENV=production

EXPOSE 3000

COPY --from=builder /code/target/release/pentair_cargo /pentair_cargo

CMD [ "/pentair_cargo" ]