import { readFileSync } from 'node:fs';
import { MockHandler } from 'vite-plugin-mock-server';
import { 
    type GuestbookEntry, 
    type Guestbook, 
    type ListRow, 
    type EntryReceipt 
} from '../static/scripts/server-types';

var guestbookDb: Guestbook = {
    guestbook: [
    {
        id: Math.random().toString(),
        timeStamp: "1993-10-02T18:37:09.030",
        name: "Linus",
        note: "I speak Malayalam now! മനുഷ്യരെല്ലാവരും തുല്യാവകാശങ്ങളോടും \
            അന്തസ്സോടും സ്വാതന്ത്ര്യത്തോടുംകൂടി ജനിച്ചിട്ടുള്ളവരാണ്‌. അന്യോന്യം \
            ഭ്രാതൃഭാവത്തോടെ പെരുമാറുവാനാണ്‌ മനുഷ്യന് വിവേകബുദ്ധിയും \
            മനസാക്ഷിയും സിദ്ധമായിരിക്കുന്നത്‌"
    },
    {
        id: Math.random().toString(),
        timeStamp: "2023-06-06T06:06:06.666",
        name: "The Devil Himself!",
        note: ""
    },
    {
        id: Math.random().toString(),
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
            res.end('17')
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
            
            if (nameLen > 150 || noteLen > 1000) { 
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
        pattern: "/lb-list-conv/conv",
        method: 'GET',
        handle: (req, res) => {
            if (req.query) {
                switch (req.query.list_name) {
                    case "server-down": 
                        res.statusCode = 502; 
                        res.statusMessage = "502 BAD GATEWAY\n";
                        break;
                    case "this-hurts-you":
                        res.statusCode = 500; 
                        res.statusMessage = "500 INTERNAL SERVER ERROR\n";
                        break;
                    case "the-big-one":
                        const textData = readFileSync("test-helpers/big-list-test.csv", {encoding: "utf8"});
                        const rowList = textData.split("\n");
                        for (const row of rowList) {
                            var lr: ListRow = {
                                totalRows: 1517,
                                rowData: row
                            }; 
                            res.write(JSON.stringify(lr));
                        }
                        break;
                    case "list-no-exist":
                        res.statusCode = 422; 
                        res.statusMessage = "422 UNPROCESSABLE CONTENT\n";
                        break;
                    case "list-too-long":
                        res.statusCode = 403; 
                        res.statusMessage = "403 FORBIDDEN\n";
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
                    res.statusCode = 422; 
                    res.statusMessage = "422 UNPROCESSABLE CONTENT\n";
                }
            }
            res.end()
        }
    }
]

export default mocks;