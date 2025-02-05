# Archie: A Self-hosting Project

I had an old HP ProBook 640 laying around, and wanted to give it new life as a web server. I got a free domain from [noip.com](https://www.noip.com/), and went from there!

I wrote a custom back-end in Rust, using the [Warp framework](https://github.com/seanmonstar/warp), as well as a small webpage for it to serve. The server runs on Arch Linux (appropriate for small projects like this), and uses a MySQL database to store data from the guestbook page.

Check it out! [archie.zapto.org](archie.zapto.org)
