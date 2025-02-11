let hitsElement = document.getElementById("hit-count");

hitsElement.innerText = "Visit count loading...";
fetch("https://archie.zapto.org/hits")
    .then(function(response) { return response.text(); })
    .then(function(respBody) {
        // double parse necessary to resolve escape sequences
        let jsonResp = JSON.parse(JSON.parse(respBody));
        hitsElement.innerText = ("Visits so far: " + jsonResp.hits_count);
    })
    .catch(error => {
        console.log(error);
        hitsElement.innerText = "(unable to get visit count)";
    }
);

