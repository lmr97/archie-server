# Archie: A Self-hosting Project

I had an old HP ProBook 640 laying around, and wanted to give it new life as a web server. I got a free domain from [noip.com](https://www.noip.com/), and went from there!

I wrote a custom back-end in Rust, using the popular [Axum framework](https://github.com/tokio-rs/axum), as well as a small webpage for it to serve. The server runs on Arch Linux (appropriate for small projects like this), using a MySQL database to store data from the guestbook page and log website hits. Both the web server component and the database run in their own Docker containers, orchestrated using `docker-compose`.

Check it out! [archie.zapto.org](archie.zapto.org)
