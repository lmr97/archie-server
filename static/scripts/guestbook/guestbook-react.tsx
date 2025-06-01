import React from 'react';
import { useState, useEffect } from 'react';
import { type Guestbook, type GuestbookEntry, type EntryReceipt } from '../server-types.ts';


export default function GuestbookApp() {

    const timeOptions: Intl.DateTimeFormatOptions = { 
        timeZone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        hour12: true,
        hour: 'numeric',
        minute: 'numeric',
        weekday: 'long', 
        year: 'numeric', 
        month: 'long', 
        day: 'numeric'
    };

    const emptyGuestbook: Guestbook = { guestbook: [] };

    // returns empty guestbook on error from server
    // same-origin creds for local testing
    const fetchGuestbook = async () => { 
        const resp = await fetch("/guestbook/entries", {credentials: "same-origin"})
            .catch(err => console.error(err));
        if (resp) {
            if (resp.ok) {
                let guestbook: Guestbook = await resp.json();
                return guestbook
            }
        } else {
            return emptyGuestbook;
        }
    };
    
    // this will allow instant updates to guestbook as it appears on the page
    const [guestbook, setGuestbook]: [Guestbook, Function] = useState(emptyGuestbook);

    // this ID is the one given by the DB
    const [latestEntryId, setLatestEntryId]: [string, Function] = useState('');
    
    useEffect(
        () => {
            console.debug("Guestbook refreshed from server.");
            fetchGuestbook()
            .then(gb => { 
                if (gb?.guestbook.length !== guestbook.guestbook.length) {
                    setGuestbook(gb)
                }
            });
        },
        [guestbook]
    );


    function GuestbookEntries() {
        if (guestbook === emptyGuestbook) {
            return <h2><i>(unable to fetch other entries)</i></h2>
        }
        const allEntries = guestbook.guestbook.map(
            (entry: GuestbookEntry) => {
                var entryDate: Date;
                if (!entry.timeStamp) {
                    entryDate = new Date();
                } else {
                    // Z, denoting UTC, gets stripped at DB. Needs to be added
                    // back in for correct parsing in JS.
                    entryDate = new Date(entry.timeStamp+"Z");
                }
                const entryTimeString = entryDate.toLocaleString("en-US", timeOptions);
                return (
                    <div className={"shine-box" + (entry.id == latestEntryId ? " your-entry": "")}>
                        <div key={entry.id} 
                            className="guestbook-entry"
                            >
                            <p className="entry entry-time">
                                {entryTimeString}
                            </p>
                            <p className="entry guest-note">
                                {entry.note}
                            </p>
                            <p className="entry guest-name">
                                — {entry.name}
                            </p>
                        </div>
                    </div>
                );
            });
        return (<>{allEntries}</>);
    }


    /* Entry Box */
    function GuestEntryForm() {

        const [charCount, setCharCount]: [number, Function] = useState(0);

        function logAlertErrorSending(errorMessage: string) {
            console.error(errorMessage);
            alert("There was an error in sending guestbook entry, sorry. \
                \nI'll look into this issue as soon as I can!");
        }

        // updates entries, locally and remotely
        function updateEntries(submitEvent: React.FormEvent<HTMLFormElement>) {

            // Prevent the browser from reloading the page,
            // and from posting the data to the current URL
            // (the default behavior)
            submitEvent.preventDefault();

            const formElement = submitEvent.currentTarget;
            const formData    = new FormData(formElement);

            // doing it this way for type correctness (and peace of mind).
            // I may change it to a simple Object.fromEntries(formData.entries()),
            // because it simply needs to be serialized.
            const newEntry: GuestbookEntry = {
                name: formData.get("name") as string,
                note: formData.get("note") as string,
            }

            fetch("/guestbook/entries", 
                {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify(newEntry),
                    credentials: "same-origin"          // for local testing
                })
                .then(resp => {
                    if (!resp.ok) {
                        throw new Error(
                            `Error status: \
                            ${resp.status} ${resp.statusText}`
                        );
                    } else {
                        return resp.json()
                    }
                })
                .then((newIdJson: EntryReceipt) => {
                    const newEntryId = newIdJson.id;
                    setLatestEntryId(newEntryId);
                })
                .catch(err => { logAlertErrorSending(`POST to server failed. ${err}`) });
            
            // THE BIG FIX: this Guestbook object needs to be set by passing 
            // in a constant or a new object literal to setGuestbook(), 
            // not a var. if it's not a const, then the rendering will be all
            // messed up.
            setGuestbook({
                guestbook: guestbook.guestbook.splice(0,0,newEntry)
            });
        }

        function countChars(keyPressEvent: React.ChangeEvent<HTMLTextAreaElement>) {
            var textGiven = keyPressEvent.target.value;
            setCharCount(textGiven.length);
        }

        return (
            <form onSubmit={updateEntries}>
                <p>
                    <label>What's your name? (you can leave it blank)  </label>
                    <input type="text" id="guestbook-name" name="name" maxLength={50} /> 
                </p>
                <p>
                    <label>What would you like to say?</label>
                    <textarea 
                        id="guestbook-note"
                        onChange={countChars} 
                        name="note" 
                        rows={4} 
                        cols={65} 
                        maxLength={1005}  // to 
                    />
                </p>
                <p className={"char-count" + (charCount > 1000 ? " too-long" : "")} >
                    Character count: {charCount} / 1000
                </p>
                <div className="buttons">
                    <button type="submit">Submit</button>
                </div>
            </form>
        )
    }


    return (
        <>
        <div className="round-box">
        <h1 id="heading"><i>Guestbook</i></h1>
            <div style={{ textAlign: "center" }}>
                <p><b>Leave a message to show you were here!</b></p>
            </div>
            <GuestEntryForm />
        </div>
        <div className="round-box" id="entry-log">
            <h2 id="heading"><i>Entries so far</i></h2>
            <GuestbookEntries />
        </div> 
        </>
    );
}

