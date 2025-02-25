let hitsElement = document.getElementById("hit-count");

hitsElement.innerText = "Visit count loading...";
fetch("https://archie.zapto.org/hits")
    .then(function(response) { return response.text(); })
    .then(function(respBody) {
        hitsElement.innerText = ("Visits so far: " + respBody);
    })
    .catch(error => {
        console.log(error);
        hitsElement.innerText = "(unable to get visit count)";
    }
);

