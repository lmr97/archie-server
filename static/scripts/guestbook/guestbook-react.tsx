import React from 'react';
import { useState, useRef, useEffect } from 'react';
import { type Guestbook, type GuestbookEntry, type EntryReceipt } from '../server-types.ts';


/* 
    Since these functions and constants don't depend on component 
    states, they can be defined outside of it. It also saves time
    and computing power, since the functions will not be redefined
    on every render.
*/

const MAX_NAME_LENGTH = 100;  // byte values
const MAX_NOTE_LENGTH = 1000;
const textEncoder = new TextEncoder();

function getDate(entry: GuestbookEntry): Date {
    
    if (!entry.timeStamp) {
        return new Date();
    } else {
        // Z, denoting UTC, gets stripped at DB. Needs to be added
        // back in for correct parsing in JS.
        return new Date(entry.timeStamp+"Z");
    }
}

function logAlertErrorSending(errorMessage: string) {
    console.error(errorMessage);
    alert("There was an error in sending guestbook entry, sorry. \
        \nI'll look into this issue as soon as I can!");
}

function displayErrorMsg(newEntry: GuestbookEntry) {
    // measures byte-length of data, courtesy of this SO post:
    // https://stackoverflow.com/questions/5515869/string-length-in-bytes-in-javascript
    
    if (textEncoder.encode(newEntry.name).length > MAX_NAME_LENGTH) {
        // this will probably not run since the HTML limits the input to MAX_NAME_LENGTH,
        // but it could, since MAX_NAME_LENGTH measures bytes, and the HTML only limits 
        // the number of "characters".
        alert("The name you entered it too long! Is there a shorter way you can identify yourself?\n\n\
            Note: The limit is 100 bytes, which can fit 100 non-accented Latin characters. \
            This limit will be reached sooner, however, with accented or non-Latin characters, \
            so you might get less than 100 characters in that case.".replaceAll("  ", ""));
        return;
    }
    if (textEncoder.encode(newEntry.note).length > MAX_NOTE_LENGTH) {
        alert("The note you entered it too long! Is there a shorter note you can leave? \
            Multiple notes are also welcome!\n\n\
            Note: The limit is 1000 bytes, which can fit 1000 non-accented Latin characters. \
            This limit will be reached sooner, however, with accented or non-Latin characters, \
            so you might get less than 1000 characters in that case.".replaceAll("  ", ""));
        return;
    }
}


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
        const resp = await fetch("/guestbook/entries")
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

    // this ID is the one given by the DB. It does not need to trigger a re-render,
    // so it can be a Ref, not a part of the state (the `guestbook` variable above 
    // is what carries the component state)
    const latestEntryId: React.RefObject<string> = useRef('');
    
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
                const entryDate: Date = getDate(entry);
                const entryTimeString = entryDate.toLocaleString("en-US", timeOptions);
                return (
                    // add "your-entry" class only if the ID matches the latest posted entry
                    <div key={entry.id} 
                        className={"shine-box" + (entry.id == latestEntryId.current ? " your-entry": "")}
                        data-testid="shine-box"
                        >
                        <div className="guestbook-entry"
                            >
                            <p className="entry entry-time" data-testid={"entry-time"+entry.id}>
                                {entryTimeString}
                            </p>
                            <p className="entry guest-note" data-testid={"entry-note"+entry.id}>
                                {entry.note}
                            </p>
                            <p className="entry guest-name" data-testid={"entry-name"+entry.id}>
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

            displayErrorMsg(newEntry);

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
                .then((newIdJson: EntryReceipt) => { latestEntryId.current = newIdJson.id; })
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
            const textGiven = keyPressEvent.target.value;
            // set at number of bytes
            setCharCount(textEncoder.encode(textGiven).length);
        }

        return (
            <form onSubmit={updateEntries}>
                <p>
                    <label>What's your name? (you can leave it blank)  </label>
                    <input 
                        type="text" 
                        id="guestbook-name" 
                        name="name" 
                        maxLength={MAX_NAME_LENGTH}
                        data-testid="name-input" 
                    /> 
                </p>
                <p>
                    <label>What would you like to say?</label>
                    <textarea 
                        id="guestbook-note"
                        onChange={countChars} 
                        name="note" 
                        rows={4} 
                        cols={65} 
                        maxLength={MAX_NOTE_LENGTH + 5}
                        data-testid="note-input"
                    />
                </p>
                <p className={"char-count" + (charCount > 1000 ? " too-long" : "")} 
                    data-testid="char-counter"
                    >
                    Note size (in bytes): {charCount} / 1000
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

