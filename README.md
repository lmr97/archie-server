# Archie: A Self-hosting Project

I had an old HP ProBook 640 laying around, and wanted to give it new life as a web server. I got a free domain from [noip.com](https://www.noip.com/), and went from there!

I wrote a custom back-end in Rust, using the popular [Axum framework](https://github.com/tokio-rs/axum), as well as a small webpage for it to serve. The server runs on Arch Linux (appropriate for small projects like this), using a MySQL database to store data from the guestbook page and log website hits. It also has [a small Python app](https://github.com/lmr97/letterboxd_get_list) that converts Letterboxd lists into CSV files. All these components run in their own Docker containers, orchestrated using Docker Compose.

Check it out! [archie.zapto.org](archie.zapto.org)

## The Grand Design: how it all works

There's a lot going on in this project, so below is a diagram of how the components relate to each other, and to the client. The subtitles in parentheses are the container names for each of the Docker containers.

![General server model](static/images/server-model.png?raw=true "General server model.")

## Client/Server/App communication protocol

Since there's a lot going on to bring the Letterboxd app together, it deserves some explanation. The process can be broken down into 4 phases, which are as follows:

### 1. Client to Server

Once the user clicks Submit, the JavaScript for the page ([`lb-conv.js`](static/scripts/lb-conv.js)) collects the list information and desired attributes, collects the list name and user who made the list out of the URL (not sending a raw URL to my server, thank you), and sends a `GET` request to `/lb-list-conv/conv`, with all all the desired info as query parameters. Then it starts listening for a stream of server-sent events. 

### 2. Server to App

The request from the client is decoded into JSON text (see example below), and sent as raw bytes to socket 3575 of the Python container (`lb-app`).

#### Conversion Request example

```
{
    "list_name": "my-super-cool-list",
    "author_user": "xXxbilly_BAxXx",
    "attrs": [ 
        "director", 
        "editor",
        "hairstyling", 
        "writer", 
	],
}
```

### 3. App to Server

The Python container reads 2048 bytes from socket 3575 (the hope is that the request won't be longer than this), and converts it into a `dict`, which it uses to assemble the list URL, and scrape the website for the requested list data. 

When the container is ready to send data back, it uses a [type-length-value encoding](https://en.wikipedia.org/wiki/Type%E2%80%93length%E2%80%93value). This encoding structures the stream as a set of frames, each with a set number of bytes containing the type and length of the payload. Since the type of data transmitted is strictly Unicode characters here, the type field is omitted, and the number of bytes to carry the payload length is set at 2. The only exception to this encoding is that, before the first row is sent, exactly 2 bytes are sent that contain the total number rows in the list (including header row). After the last row is transmitted, process completion is signaled by one final frame with length bytes set to `0x00 0x05`, and the payload being the string literal `done!`.

### 4. Server to Client

After the server recieves a row (and decodes it), it bundles it up as a server-sent event ([`axum::response::sse::Event`](https://docs.rs/axum/latest/axum/response/sse/struct.Event.html)), and sends it off to the client, which has been listening for such this whole time. For an example of the payload this event carries, see below (the payload is JSON-formatted text). When a frame with a `done!` payload is received by server from the Python container, an event of type `complete` is sent to the client, and when it is received by the client, the client closes out the connection.

#### Event payload example

```
{
    "curr_row": 4,
    "total_rows": 45,
    "row": "\"Sorry to Bother You\",2018,\"Boots Riley\",\"Terel Gibson\",\"Antionette Yoka\",\"Boots Riley\"",
}
```
Note that this always includes a title and year entry in the row, in addition to the attributes requested. So if no list attributes are specified, title and year alone will be sent. Each non-numeric field is also quote-enclosed to make the CSV file parse correctly.

## Local Demo

Since it's all Dockerized, you can also spin it up locally! If you have Docker running on your system, and Docker Compose installed, all you need to do is:

```
git clone https://github.com/lmr97/archie-server
cd archie-server
git submodule update --init --recursive
npm install
docker compose --file compose-demo.yaml up --detach
```

It'll probably take a while to build the Docker images (it takes ~5 minutes total on my machine, the majority of which was for the central server image). And once the containers are started, give the database container ~2 minutes to initialize before trying it out (otherwise there will be errors). You can see if the database is ready by running `docker container ls` looking for whether the `archie-db` container is marked healthy or not.

Now you can try it out! The server is listening at `localhost:3000`.