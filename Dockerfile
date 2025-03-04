# to build (an image named "archie", from repo root):
# sudo docker build -t archie .
#
# to run (on port 443):
# sudo docker run -d -p 4949:443 archie

# this could be a multi-stage build, but I'm keeping it
# single-stage because of Arch's rolling-release model,
# and I want to ensure all the package updates are in sync
# with each other and the current Arch version.
FROM archlinux

RUN pacman-key --init
RUN pacman -Sy && \
    pacman -S --noconfirm \
        augeas \
        cronie \
        git \
        mariadb \ 
        python \
        rust   

# set up server user account
RUN useradd server
USER server
# keys are copied in, since they need to be 
# updated on the server itself
WORKDIR /home/server
COPY --chown=server ./fullchain.pem ./privkey.pem ./
RUN touch archie-server.log
ENV HOME          "/home/server"
ENV CRT_FILE      "$HOME/fullchain.pem"
ENV PK_FILE       "$HOME/privkey.pem"
ENV DB_URL        "mysql://server:s3rv3r-pass@localhost:3306/archie"
ENV SERVER_LOG    "$HOME/archie-server.log"
ENV SERVER_ROOT   "$HOME/archie-server"
ENV SERVER_SOCKET "0.0.0.0:4949"

# using git instead of COPY so that updates can be applied from
# within the container, and won't require rebuilding the image
RUN git clone https://github.com/lmr97/archie-server
WORKDIR "$HOME/archie-server/custom-backend"
RUN cargo build --release
# allows new builds to not overwrite the last build
RUN cp ./target/release/custom-backend ~/archie-svr 

EXPOSE 4949
CMD [ "/home/server/archie-svr" ]