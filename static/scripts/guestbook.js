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


function updateGuestbookRemote() {
    let guest = document.getElementById("guestbook-name").value;
    let note  = document.getElementById("guestbook-note").value;
    
    let newEntry = {
        "name": guest,
        "note": note
    }

    if (newEntry.name.length == 0) newEntry.name = "(anonymous)";

    fetch(window.location.href+"/entries", {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(newEntry)
      }
    );
    
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


function updateGuestbookDisplay() {
    
    fetch(window.location.href+"/entries")
        .then(
            response => response.text()
        ).then(
            respBody => {
                let allEntries = JSON.parse(respBody)["guestbook"];
                let entryDisplay = document.getElementById("entry-log");

                for (let i = 0; i < allEntries.length; i++) {
                    let entryElement = populateEntry(allEntries[i.toString()]);
                    entryDisplay.appendChild(entryElement);
                }
            }
        );
}


updateGuestbookDisplay();
document.querySelector("button").addEventListener("click", updateGuestbookRemote);
