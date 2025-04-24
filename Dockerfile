# to build (an image named "archie-svr", from repo root):
# sudo docker build -t archie-svr .

# WARNING: this will build, but not run as such without 
# secrets being manually mounted, when using the default 
# compose.yaml. For this reason, a compose-demo.yaml has been
# added for demo purposes.


# Oddly, the method below is the only known way to cache dependency  
# builds for Rust Docker images. It allows the dependencies and 
# `main.rs` to be built using separate commands, so the dependency  
# build command can be cached. 
#
# It works by switching the real `main.rs` for a dummy `main.rs` 
# with an empty `main` function in the Manifest, building the 
# dependencies, substituting the real `main.rs` afterwards. 
# Only then does it build the whole project -- with the dependencies 
# already found to be built. 
#
# This behavior can obviously be overriden with the `--no-cache` 
# option with `docker build`.
#
# This method comes from this blog post:
# https://web.archive.org/web/20221028051630/https://blog.mgattozzi.dev/caching-rust-docker-builds/
# 
# Found through this StackOverflow thread:
# https://stackoverflow.com/questions/58473606/cache-rust-dependencies-with-docker-build

FROM rust:1.83.0

RUN echo "fn main() {}" > dummy.rs
COPY ./custom-backend/Cargo.toml .
COPY ./custom-backend/Cargo.lock .
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml
COPY ./custom-backend .
RUN cargo build --release

ENV SERVER_ROOT="/home/server"
ENV SERVER_LOG="$SERVER_ROOT/archie-server.log"
ENV SERVER_SOCKET="0.0.0.0:4949"
ENV CRT_FILE="/run/secrets/server-cert"
ENV PK_FILE="/run/secrets/server-priv-key"

# make sure the log is fresh for the image
RUN touch "./archie-server.log"

EXPOSE 4949
CMD [ "./target/release/archie-server" ]