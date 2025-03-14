FROM rust:latest as base

RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown

FROM base as devcontainer
RUN cargo install leptosfmt