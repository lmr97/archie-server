# The frequent use of the `chown` option on COPY is to make sure
# the image/container has permissions to the files when running
# in rootless mode


###########################################
#                                         #
#     STAGE 1: Build Rust dependencies    #
#                                         #
###########################################
FROM rust:1.88.0 AS deps

RUN useradd -m server
USER server

WORKDIR /home/server/custom-backend
RUN echo "fn main() {}" > dummy.rs
COPY --chown=server:server ./custom-backend/Cargo.toml ./
COPY --chown=server:server ./custom-backend/Cargo.lock ./
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release



###########################################
#                                         #
#     STAGE 2: Build server executable    #
#                                         #
###########################################
FROM rust:1.88.0 AS main-build

# Install Node, for ViteJS integration. It's easier to set up Node 
# from a Rust environment than the other way around, I've found. 
RUN curl -fsSL https://deb.nodesource.com/setup_24.x -o nodesource_setup.sh
RUN bash nodesource_setup.sh
RUN apt-get install -y nodejs

RUN useradd -m server
USER server
WORKDIR /home/server

# In order to use Vite with a Rust backend (using vite-rs), we've 
# gotta have EVERYTHING in the image for building, so it's all gotta be here...

# ...the packages...
COPY --chown=server:server ./package.json      ./
COPY --chown=server:server ./package-lock.json ./
RUN npm install

# ...and the content.
COPY --chown=server:server ./pages      ./pages/
COPY --chown=server:server ./static     ./static/
COPY --chown=server:server ./index.html ./

# since the assets are built into the server binary, the Vite config file
# should be statically copied into the image, not mounted 
COPY --chown=server:server ./vite.config.ts ./

# Finally, build the server executable
WORKDIR /home/server/custom-backend
COPY --chown=server:server ./custom-backend ./
# target directory is ignored in .dockerignore
COPY --from=deps --chown=server:server \
    /home/server/custom-backend/target/release \
    ./target/release/
RUN cargo build --release



###########################################
#                                         #
#   STAGE 3: Finalize server environment  #
#                                         #
###########################################
FROM debian AS server-run

# install cURL for healthcheck
RUN apt-get update
RUN apt-get install -y curl

# don't run as root inside container
RUN useradd -m server
USER server

WORKDIR /home/server
COPY --from=main-build --chown=server:server \
    /home/server/custom-backend/target/release/archie-server \
    ./custom-backend/target/release/archie-server
COPY --from=main-build --chown=server:server \
    /home/server/dist ./dist/

# set up environment variables that will always
# be the same relative to the server. The ones that
# may change (or depend on host environment) are 
# in the Compose file.
ENV SERVER_ROOT="/home/server"
ENV SERVER_LOG="/home/server/archie-server.log"
ENV SERVER_SOCKET="0.0.0.0:4949"
ENV CRT_FILE="/run/secrets/server-cert"
ENV PK_FILE="/run/secrets/server-priv-key"


# make sure the log is fresh for the image
RUN touch "./archie-server.log"

WORKDIR /home/server/custom-backend
EXPOSE 4949
CMD [ "/home/server/custom-backend/target/release/archie-server" ]
