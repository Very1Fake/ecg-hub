FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR app

####################################

FROM chef AS planner
COPY . .
RUN cargo chef prepare

####################################

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release
COPY . .
RUN cargo build --release -p ecg-hub

####################################

FROM gcr.io/distroless/cc AS runtime

COPY --from=builder /app/target/release/ecg-hub /usr/local/bin

CMD ["/usr/local/bin/ecg-hub"]
