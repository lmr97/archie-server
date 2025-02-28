let hitsElement = document.getElementById("hit-count");
hitsElement.innerText = "Visit count loading...";

const hit = {
    time_stamp: new Date().toISOString().slice(0, -1),  // shave off Z
    user_agent: navigator.userAgent
};

const options = {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(hit)
};

// POST hit from initial page load...
fetch(window.location.href + "hits", options)
    .then(resp => {
        if (!resp.ok) throw new Error(
            `POST hit to /hits failed with status code ${resp.status}`
        );
    })
    .catch(error => console.log(error));

// ...then GET total hits
fetch(window.location.href + "hits")
    .then((resp) => { 
        if (!resp.ok) throw new Error(
             `GET /hits failed with status code ${resp.status}`
        );
        return resp.text();
    })
    .then(result => 
        hitsElement.innerText = `Visits so far: ${result}`
    )
    .catch(error => {
        console.log(error);
        hitsElement.innerText = "(unable to get visit count)";
    }
);

