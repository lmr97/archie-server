import { readFileSync } from 'node:fs';
import { MockHandler } from 'vite-plugin-mock-server';
import { createSession, Session } from 'better-sse';
import { 
    type GuestbookEntry, 
    type Guestbook, 
    type ListRow, 
    type EntryReceipt 
} from '../../static/scripts/server-types';


var guestbookDb: Guestbook = {
    guestbook: [
    {
        id: "0.0051166112644069894",
        timeStamp: "1993-10-02T18:37:09.030",
        name: "Linus",
        note: "I speak Malayalam now! മനുഷ്യരെല്ലാവരും തുല്യാവകാശങ്ങളോടും \
            അന്തസ്സോടും സ്വാതന്ത്ര്യത്തോടുംകൂടി ജനിച്ചിട്ടുള്ളവരാണ്‌. അന്യോന്യം \
            ഭ്രാതൃഭാവത്തോടെ പെരുമാറുവാനാണ്‌ മനുഷ്യന് വിവേകബുദ്ധിയും \
            മനസാക്ഷിയും സിദ്ധമായിരിക്കുന്നത്‌"
    },
    {
        id: "0.7486503970331404",
        timeStamp: "2023-06-06T06:06:06.666",
        name: "The Devil Himself!",
        note: ""
    },
    {
        id: "0.6871818383290702",
        timeStamp: "2099-03-06T01:12:00.232",
        name: "(anonymous)",
        note: "You won't even known who this is..."
    },
]};

// don't forget to export `mocks`! see the last line
const mocks: MockHandler[] = [
    {
        pattern: "/hits", 
        method: 'GET',
        handle: (_req, res) => {
            res.end('8002934')
        }
    },
    {
        pattern: "/hits", 
        method: 'POST',
        handle: (req, res) => {
            res.end("Thanks for stopping by!\n");
        }
    },
    {
        pattern: "/guestbook", 
        method: 'GET',
        handle: async (_req, res) => {
            const pageData: string = await fetch("http://localhost:5173/pages/guestbook.html")
                .then(resp => {return resp.text()});
            res.end(pageData); 
        }
    },
    {
        pattern: "/guestbook/entries", 
        method: 'GET',
        handle: (_req, res) => {
            res.end(JSON.stringify(guestbookDb)); 
        }
    },
    {
        pattern: "/guestbook/entries", 
        method: 'POST',
        handle: (req, res) => {

            var newEntry: GuestbookEntry = req.body;

            // from this SO post: https://stackoverflow.com/questions/5515869/string-length-in-bytes-in-javascript
            // For measuring the length in bytes
            const textEncoder = new TextEncoder();
            const nameLen = textEncoder.encode(newEntry.name).length;
            const noteLen = textEncoder.encode(newEntry.note).length;
            
            if (nameLen > 100 || noteLen > 1000) { 
                res.statusCode = 413;
                res.statusMessage = "413 PAYLOAD TOO LARGE\n"
                res.end()
            } else {
                console.log(`New entry from: ${newEntry.name}`)
                newEntry.name      = newEntry.name ? newEntry.name : "(anonymous)"
                newEntry.timeStamp = (new Date()).toISOString().slice(0,-1);
                newEntry.id        = Math.random().toString();
                guestbookDb.guestbook.splice(0,0,newEntry);

                const receipt: EntryReceipt = {
                    timeStamp: newEntry.timeStamp,
                    // yes, it's a float, but it's deserialized as a string anyway 
                    id: newEntry.id
                };
                res.end(JSON.stringify(receipt));
            }
        }
    },
    {
        pattern: "/lb-list-conv", 
        method: 'GET',
        handle: async (_req, res) => {
            const pageData: string = await fetch("http://localhost:5173/pages/lb-list-app.html")
                .then(resp => {return resp.text()});
            res.end(pageData); 
        }
    },
    {
        pattern: "/lb-list-conv/conv",
        method: 'GET',
        handle: async (req, res) => {
            
            if (!req.query) {
                res.statusCode = 400; 
                res.statusMessage = "400 BAD REQUEST\n";
                res.end();
                return;
            }

            const sseEmitter: Session = await createSession(req, res);

            switch (req.query.list_name) {
                case "server-down": 
                    sseEmitter.push("502 BAD GATEWAY\n", "error");
                    break;
                case "this-hurts-you":
                    sseEmitter.push("500 INTERNAL SERVER ERROR\n", "error");
                    break;
                case "the-big-one":
                    const textData = readFileSync("../big-list-test.csv", {encoding: "utf8"});
                    const rowList = textData.split("\n");
                    for (const row of rowList) {
                        var lr: ListRow = {
                            totalRows: 1517,
                            rowData: row
                        }; 
                        sseEmitter.push(JSON.stringify(lr));
                    }
                    sseEmitter.push("done!", "complete");
                    break;
                case "list-no-exist":
                    sseEmitter.push("422 UNPROCESSABLE CONTENT\n", "error");
                    break;
                case "list-too-long":
                    sseEmitter.push("403 FORBIDDEN\n", "error");
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

                sseEmitter.push(JSON.stringify(header));
                for (var i = 0; i < 4; i++) {
                    
                    var lr: ListRow = {
                        totalRows: 4,
                        rowData: `${titles[i]},${years[i]}`
                    }; 
                    sseEmitter.push(JSON.stringify(lr));
                }
                sseEmitter.push("done!", "complete");
            } 
            else if (req.query.attrs.includes("bingus")) {
                sseEmitter.push("422 UNPROCESSABLE CONTENT\n", "error");
            }
        }
    }
]

export default mocks;