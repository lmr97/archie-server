# to build (an image named "archie", from repo root):
# sudo docker build -t archie .
#
# to run (on host port 443):
# sudo docker run -d -p 443:4949 archie

FROM rust

RUN useradd server
USER server
WORKDIR /home/server
COPY --chown=server . .

ENV HOME          "/home/server"
ENV CRT_FILE      "$HOME/fullchain.pem"
ENV PK_FILE       "$HOME/privkey.pem"
ENV DB_URL        "mysql://server1:s3rv3r-pass@db:33061/archie"
ENV SERVER_LOG    "$HOME/archie-server.log"
ENV SERVER_ROOT   "$HOME/archie-server"
ENV SERVER_SOCKET "0.0.0.0:4949"

RUN touch "$HOME/archie-server.log"

WORKDIR "$HOME/custom-backend"
RUN cargo install --path .

EXPOSE 4949
CMD [ "custom-backend" ]