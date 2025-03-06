# to build (an image named "archie", from repo root):
# sudo docker build -t archie .
#
# to run (on host port 443):
# sudo docker run -d -p 443:4949 archie

FROM rust:1.83.0

RUN useradd server
USER server
WORKDIR /home/server
COPY --chown=server . .

ENV HOME          "/home/server"
ENV CRT_FILE      "/run/secrets/server-cert"
ENV PK_FILE       "/run/secrets/server-priv-key"
ENV SERVER_LOG    "$HOME/archie-server.log"
ENV SERVER_ROOT   "$HOME/archie-server"
ENV SERVER_SOCKET "0.0.0.0:4949"

RUN touch "$HOME/archie-server.log"

WORKDIR "$HOME/custom-backend"
RUN cargo install --path .

EXPOSE 4949
CMD [ "custom-backend" ]