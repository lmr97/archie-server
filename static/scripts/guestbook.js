function updateGuestbookRemote() {
    let guest = document.getElementById("guestbook-name").value;
    let note  = document.getElementById("guestbook-note").value;
    
    fetch('https://archie.zapto.org/guestbook/entries', {
        method: 'POST',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(
            {
                "name": guest,
                "note": note
            }
        )
      }
    );
    
    // reset fields
    document.getElementById("guestbook-name").value = "";
    document.getElementById("guestbook-note").value = "";

    // update list of entries without reloading page
    updateGuestbookDisplay();
}

function populateEntry(entryData, timeOpts) {
    
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
            child.textContent = dateTime.toLocaleString("en-US", timeOpts);
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
    
    fetch("https://archie.zapto.org/guestbook/entries")
        .then(
            response => response.text()
        ).then(
            respBody => {
                // the double JSON parsing accounts for literal backslashes in
                // the response body. The first parse renders the escaped double-quotes,
                // and the second one actually deserializes.
                let allEntries = JSON.parse(JSON.parse(respBody));
                let entryDisplay = document.getElementById("entry-log");
                
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

                for (let i = 0; i < allEntries.length; i++) {
                    let entryElement = populateEntry(allEntries[i.toString()], timeOptions);
                    entryDisplay.appendChild(entryElement);
                }
            }
        );
}

updateGuestbookDisplay();
document.querySelector("button").addEventListener("click", updateGuestbookRemote);