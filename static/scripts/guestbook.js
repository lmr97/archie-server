import { parse } from '/node_modules/@vanillaes/csv/index.js'

function updateGuestbookRemote() {
    let guest = document.getElementById("guestbook-name").value;
    let note  = document.getElementById("guestbook-note").value;
    
    let xhr = new XMLHttpRequest();

    xhr.open("POST", "http://localhost:8080/guestbook-entries", true);
    xhr.setRequestHeader("Content-Type", "application/json");
    xhr.send(
        JSON.stringify(
            {
                "name": guest,
                "note": note
            }
        )
    );

    // reset fields
    document.getElementById("guestbook-name").value = "";
    document.getElementById("guestbook-note").value  = "";
}

function populateEntry(entryData) {
    // make new entry node
    console.log(entryData);
    let entryElement = document.createElement("div");
    entryElement.classList.add("guestbook-entry");

    // doing this as a loop allows for easy extensibility
    let childrenClasses = ["entry-time", "entry-text", "guest-name"];
    for (let i = 0; i < childrenClasses.length; i++) {

        let child = document.createElement("p");
        child.classList.add("entry");
        child.classList.add(childrenClasses[i]);

        // add em-dash to start of guest name only
        if (i == 2) {
            child.textContent = "— ";
        }
        child.textContent += entryData[i];

        entryElement.appendChild(child);
    }
    
    return entryElement;
}

function updateGuestbookDisplay() {
    
    fetch("http://localhost:8080/guestbook.csv")
        .then(
            response => response.text()
        ).then(
            respBody => {
                let allEntries = parse(respBody);
                console.log(allEntries);
                let entryDisplay = document.getElementById("entry-log");

                for (let i = 0; i < allEntries.length; i++) {
                    let entryElement = populateEntry(allEntries[i]);
                    entryDisplay.appendChild(entryElement);
                }
            }
        );
}

updateGuestbookDisplay();
document.querySelector("button").addEventListener("click", updateGuestbookRemote);