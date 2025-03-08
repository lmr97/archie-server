let timeOptions = { 
    timeZone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    hour12: true,
    hour: 'numeric',
    minute: 'numeric',
    weekday: 'long', 
    year: 'numeric', 
    month: 'long', 
    day: 'numeric' 
}


async function updateGuestbookRemote() {
    let guest = document.getElementById("guestbook-name").value;
    let note  = document.getElementById("guestbook-note").value;
    
    // length restrictions are emforced by textarea element itself

    let newEntry = {
        "name": guest,
        "note": note
    }

    if (newEntry.name.length == 0) newEntry.name = "(anonymous)";

    try {
        // catch exceptions thrown by fetch()
        const response = await fetch(window.location.href+"/entries", 
            {
                method: 'POST',
                headers: {
                'Content-Type': 'application/json'
                },
                body: JSON.stringify(newEntry)
            }
        );

        // catch server error responses
        if (!response.ok) {
            throw new Error(
                `POST entry to db failed with status code: ${resp.status}`
            );
        }
    }
    catch (err) {
        console.error(`POST entry to db failed, because: ${err}`);
        alert("Error in recieving guestbook entry, sorry. \nI'll look into this issue as soon as I can!");
        return;
    }
    
    // reset fields
    document.getElementById("guestbook-name").value = "";
    document.getElementById("guestbook-note").value = "";
    
    alert("Entry recieved. Thank you for leaving a note!");

    // add entry to page, at the top of the list
    newEntry["time_stamp"] = new Date().toISOString().slice(0, -1);  // shave off Z

    const entryNode        = populateEntry(newEntry);
    let entries            = document.getElementsByClassName("guestbook-entry");
    entries[0].before(entryNode);
}


function populateEntry(entryData) {
    
    // make new entry node
    let entryElement = document.createElement("div");
    entryElement.classList.add("guestbook-entry");

    // this maps the JSON object keys to the HTML classes 
    // for the page
    let keysToClasses = {
        time_stamp: "entry-time", 
        note: "guest-note",
        name: "guest-name"
    };

    // this keeps track of the p elements,
    // so I can append them to the div in a specific order
    let entryObject = new Object();

    for (const key in entryData) {

        let child = document.createElement("p");
        child.classList.add("entry");            // the common class
        child.classList.add(keysToClasses[key]);

        // add em-dash to start of guest name only
        if (key == "name") {
            child.textContent = "— ";
            child.textContent += entryData[key];
        } else if (key == "time_stamp") {
            var dateTime = new Date(entryData[key]+"Z");  // the Z indicates UTC 
            child.textContent = dateTime.toLocaleString("en-US", timeOptions);
        } else {
            child.textContent += entryData[key];
        }
        
        entryObject[key] = child;
    }

    entryElement.appendChild(entryObject["time_stamp"]);
    entryElement.appendChild(entryObject["note"]);
    entryElement.appendChild(entryObject["name"]);
    
    return entryElement;
}


function displayCharCount(guestbookNoteElement) {
    let countElement = document.getElementById("char-count");
    let entryLength = guestbookNoteElement.value.length;

    countElement.textContent = `Charcter count: ${entryLength}/1000`;

    // using >= cuz you never know
    if (entryLength >= 1000) {
        countElement.textContent = "Charcter count: 1000/1000 (maximum reached)";
    }
}

function updateGuestbookDisplay() {
    
    fetch(window.location.href+"/entries")
        .then(
            response => {
                if (!response.ok) {
                    throw new Error(
                        `Error in retrieving entries. Status code: ${response.status}`
                    )
                } 
                return response.text();
            }
        ).then(
            respBody => {
                let allEntries = JSON.parse(respBody)["guestbook"];
                let entryDisplay = document.getElementById("entry-log");

                for (let i = 0; i < allEntries.length; i++) {
                    let entryElement = populateEntry(allEntries[i.toString()]);
                    entryDisplay.appendChild(entryElement);
                }
            }
        ).catch(
            err => {
                let entryDisplay = document.getElementById("entry-log");

                let pageErrElement = document.createElement("p");
                pageErrElement.textContent = "(cannot retrieve other entries)";
                pageErrElement.style["font-style"] = "italic";

                entryDisplay.appendChild(pageErrElement);
                console.error(err);
            }
        );
}


updateGuestbookDisplay();

let noteEntryArea = document.getElementById("guestbook-note");
noteEntryArea.addEventListener("keydown", function() {displayCharCount(this)});
noteEntryArea.addEventListener("keyup",   function() {displayCharCount(this)});

document
    .querySelector("button")
    .addEventListener(
        "click", updateGuestbookRemote
    );