# to build (an image named "archie", from repo root):
# sudo docker build -t archie .

# WARNING: this will build, but not run as such without 
# secrets being manually mounted (the image is meant for a 
# Docker Compose run)

FROM rust:1.83.0

RUN useradd server
USER server
WORKDIR /home/server
COPY ./custom-backend .

ENV SERVER_ROOT="/home/server"
ENV SERVER_LOG="$SERVER_ROOT/archie-server.log"
ENV SERVER_SOCKET="0.0.0.0:4949"
ENV CRT_FILE="/run/secrets/server-cert"
ENV PK_FILE="/run/secrets/server-priv-key"

# make sure the log is fresh for the image
RUN touch "./archie-server.log"

RUN cargo build --release

EXPOSE 4949
CMD [ "./target/release/custom-backend" ]