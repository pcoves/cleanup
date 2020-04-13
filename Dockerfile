FROM rust as build
COPY . /cleanup
RUN cargo install --path /cleanup

FROM gcr.io/distroless/cc
COPY --from=build /usr/local/cargo/bin /usr/local/cargo/bin
ENTRYPOINT ["/usr/local/cargo/bin/cleanup"]
