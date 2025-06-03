let hitsElement = document.getElementById("hit-count");
hitsElement.innerText = "Visit count loading...";

const hit = {
    time_stamp: new Date().toISOString().slice(0, -1),  // shave off Z
    user_agent: navigator.userAgent
};

const postOptions = {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(hit),
    credentials: "same-origin"          // for local testing
};

async function updateHits() {
    // POST hit from initial page load...
    await fetch("/hits", postOptions)
        .then(resp => {
            if (!resp.ok) throw new Error(
                `POST hit to /hits failed with status code ${resp.status}`
            );
        })
        .catch(error => console.log(error));

    // ...then GET total hits
    await fetch("/hits", {credentials: "same-origin"})
        .then((resp) => { 
            if (!resp.ok) throw new Error(
                `GET /hits failed with status code ${resp.status}`
            );
            return resp.text();
        })
        .then(result => {
            const fmtNumber = Number(result).toLocaleString();
            hitsElement.innerText = `Visits so far: ${fmtNumber}`;
        })
        .catch(error => {
            console.log(error);
            hitsElement.innerText = "(unable to get visit count)";
        }
    );  
}

updateHits();


