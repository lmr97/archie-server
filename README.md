# Archie: A Self-hosting Project

I had an old HP ProBook 640 laying around, and wanted to give it new life as a web server. I got a free domain from [noip.com](https://www.noip.com/), and went from there!

I wrote a custom back-end in Rust, using the popular [Axum framework](https://github.com/tokio-rs/axum), as well as a small webpage for it to serve. The server runs on Arch Linux (appropriate for small projects like this), using a MySQL database to store data from the guestbook page and log website hits. Both the web server component and the database run in their own Docker containers, orchestrated using Docker Compose.

Check it out! [archie.zapto.org](archie.zapto.org)

## Local Demo

Since it's all Dockerized, you can also spin it up locally! If you have Docker running on your system, and Docker Compose installed, all you need to do is:

1. Clone the repo, of course:
```
git clone https://github.com/lmr97/archie-server
cd archie-server
```

2. Set a `MYSQL_PASSWORD` environment variable:
```
export MYSQL_PASSWORD=whatever-you-like
```

3. Spin up the containers:
```
sudo --preserve-env \
    docker compose \
    --file compose-demo.yaml \
    up -d
```
It'll probably take a while to build the images (it took ~5 minutes total on my machine, 3 of which was for the central server image). And once the containers are started, give the database container ~2 minutes to initialize before trying it out. You can see if it's ready for connections by running `sudo docker logs archie-db`. 

4. Try it out! You can reach the server at `localhost:3000`.