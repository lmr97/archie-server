# Archie: A Self-hosting Project

I had an old HP ProBook 640 laying around, and wanted to give it new life as a web server. I got a free domain from [noip.com](https://www.noip.com/), and went from there!

I wrote a custom back-end in Rust, using the popular [Axum framework](https://github.com/tokio-rs/axum), as well as a small webpage for it to serve. The server runs on Arch Linux (appropriate for small projects like this), using a MySQL database to store data from the guestbook page and log website hits. It also has [a small Python app](https://github.com/lmr97/letterboxd_get_list) that converts Letterboxd lists into CSV files. All these components run in their own Docker containers, orchestrated using Docker Compose.

Check it out! [archie.zapto.org](archie.zapto.org)

## The Grand Design: how it all works

There's a lot going on in this project, so below is a diagram of how the components relate to each other, and to the client. The subtitles in parentheses are the container names for each of the Docker containers.

![General server model](static/images/server-model.png?raw=true "General server model.")

### Server/App communications protocol

Since there isn't any readily available API for a Rust program and a Python program to communicate over a network (to my knowledge), I needed to define a structure to a raw byte stream that could be passed back and forth between the two containers. It goes as follows:

![Letterboxd app protocol](static/images/lb-app-model.png?raw=true "Letterboxd app communications protocol.")

When the scerver gets a row, it bundles it up as a server-sent event ([`axum::response::sse::Event`](https://docs.rs/axum/latest/axum/response/sse/struct.Event.html)), and send it off to the client, which listens for events after sending a `GET` request. For an example of the payload this event carries, see below. The "done!" literal sent at the end is bundled up with an event of type `complete`, and when it is received by the client, the client closes out the connection.

#### Conversion Request example
```
{
	"list_name": "my-super-cool-list",
	"author_user": "xXxbilly_BAxXx",
	"attrs": [ 
			"director" "editor",
			"hairstyling", "writer", 
	],
}
```

#### Event payload example
```
{
    "curr_row": 4,
    "total_rows": 45,
    "row": "\"Sorry to Bother You\",2018,Boots Riley,Terel Gibson,Antionette Yoka,Boots Riley",
}
```
Note that this always includes a title and year entry in the row, in addition to the attributes requested.

## Local Demo

Since it's all Dockerized, you can also spin it up locally! If you have Docker running on your system, and Docker Compose installed, all you need to do is:

```
git clone https://github.com/lmr97/archie-server
cd archie-server
git submodule update --init --recursive
npm install
docker compose \   
    --file compose-demo.yaml \
    up --detach
```
It'll probably take a while to build the images (it took ~5 minutes total on my machine, 3 of which was for the central server image). And once the containers are started, give the database container ~2 minutes to initialize before trying it out (otherwise there will be errors). You can see if the database is ready for connections by running `docker logs archie-db`. 

4. Try it out! You can reach the server at `localhost:3000`.