// exists to emulate the server's back-end API for testing,
// while forwarding all static requests to Vite's dev server.
import fs from 'node:fs';
import express from 'express';  // missing, needs reimport
import { parse } from 'qs';
import { type Guestbook, type ListRow } from '../static/scripts/server-types.ts';
import request from 'request';  // missing, needs reimport

function viteRedirect(req, res) {
  var url = "http://localhost:5173" + req.url;
  console.log(`Redirect URL: ${url}`);
  req.pipe(request(url)).pipe(res);
}

const server = express()
server.use(express.json())
server.enable('trust proxy')
server.use('/static', (req, res) =>  {
  viteRedirect(req, res)
});
server.use('/', (req, res) =>  {
  viteRedirect(req, res)
});
server.set('query parser',
  (str) => parse(str, { duplicates: 'combine' })
)

server.get("/hits", (_req, res) => {
  res.send('17')
})


server.post("/hits", (req, res) => {
  console.log(`Received a hit from ${req.body.user_agent} at ${req.body.time_stamp}!`)
  res.send("Thanks for your entry!\n")
})

server.get("/guestbook/entries", (_req, res) => {

  var gb: Guestbook = {
    guestbook: [
      {
        timeStamp: "1993-10-02T18:37:09.030",
        name: "Linus",
        note: "I speak Malayalam now! മനുഷ്യരെല്ലാവരും തുല്യാവകാശങ്ങളോടും \
              അന്തസ്സോടും സ്വാതന്ത്ര്യത്തോടുംകൂടി ജനിച്ചിട്ടുള്ളവരാണ്‌. അന്യോന്യം \
              ഭ്രാതൃഭാവത്തോടെ പെരുമാറുവാനാണ്‌ മനുഷ്യന് വിവേകബുദ്ധിയും \
              മനസാക്ഷിയും സിദ്ധമായിരിക്കുന്നത്‌"
      },
      {
        timeStamp: "2023-06-06T06:06:06.666",
        name: "The Devil Himself!",
        note: ""
      },
      {
        timeStamp: "2099-03-06T01:12:00.232",
        name: "(anonymous)",
        note: "You won't even known who this is..."
      },
    ]
  }

  res.json(gb);
})

server.post("/guestbook/entries", (req, res) => {
  // from this SO post: https://stackoverflow.com/questions/5515869/string-length-in-bytes-in-javascript
  // For measuring the length in bytes
  const textEncoder = new TextEncoder();
  var nameLen = textEncoder.encode(req.body.name).length;
  var noteLen = textEncoder.encode(req.body.note).length;

  if (nameLen > 150 || noteLen > 1000) { 
    res.status(413).send("413 PAYLOAD TOO LARGE\n");
  } else {
    console.log(`New entry from: ${req.body.name}`)
    res.send("Entry received. Thanks!\n")
  }
})

server.get("/lb-list-conv/conv", (req, res) => {

  switch (req.query.list_name) {
    case "server-down": 
      res.status(502).send("502 BAD GATEWAY\n");
      break;
    case "this-hurts-you":
      res.status(500).send("500 INTERNAL SERVER ERROR\n");
      break;
    case "the-big-one":
      const textData = fs.readFileSync("test-helpers/big-list-test.csv", {encoding: "utf8"});
      const rowList = textData.split("\n");
      for (const row of rowList) {
        var lr: ListRow = {
          totalRows: 1517,
          rowData: row
        }; 
        res.write(JSON.stringify(lr));
      }
      res.end();
      break;
    case "list-no-exist":
      res.status(422).send("422 UNPROCESSABLE CONTENT\n");
      break;
    case "list-too-long":
      res.status(403).send("403 FORBIDDEN");
      break;
  }

  // list with only the none attribute are parsed as 
  if (req.query.attrs == 'none') {

    var titles = ["2001: A Space Odyssey", "Blade Runner",
        "The Players vs. Ángeles Caídos", "8½"]
    var years  = ["1968", "1982", "1969", "1963"]
    var header: ListRow = {
        totalRows: 4,
        rowData: "Title,Year"
      }; 
    res.write(JSON.stringify(header));
    for (var i = 0; i < 4; i++) {
      var lr: ListRow = {
        totalRows: 4,
        rowData: `${titles[i]},${years[i]}`
      }; 
      res.write(JSON.stringify(lr));
    }
  } 
  else if (req.query.attrs.includes("bingus")) {
    res.status(422).send("422 UNPROCESSABLE CONTENT\n");
  }
  res.end()
})


var port = 8080;
server.listen(port, () => {
  console.log(`\nMock backend listening on port ${port}...`)
})

