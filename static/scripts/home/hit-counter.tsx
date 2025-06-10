import React from 'react';
import { useState, useRef, useEffect } from 'react';
import { type WebpageHit } from '../server-types.ts';

export default function HitCounter() {

    const hit: WebpageHit = {
        timeStamp: new Date().toISOString().slice(0, -1),  // shave off Z
        userAgent: navigator.userAgent
    };

    const postOptions = {
        method: 'POST',
        headers: {
        'Content-Type': 'application/json'
        },
        body: JSON.stringify(hit),
    };

    const hitPosted: React.RefObject<boolean> = useRef(false);

    //   0: void state (count not retrieved)
    //  -1: error state
    // > 0: normal state
    const [hits, setHits]: [number, Function] = useState(0);

    useEffect(() => {
            if (!hitPosted.current) {
                hitPosted.current = true;
                postHit();
            }
            getHits()
                .then((hitCount: number) => { 
                    if (hitCount > hits) {
                        setHits(hitCount)
                    }
                });
        },
        [hits]
    )

    function postHit(): void {
        fetch(window.location.origin+"/hits", postOptions)
            .then(resp => {
                if (!resp.ok) throw new Error(
                    `POST hit to /hits failed with status code ${resp.status}`
                );
            })
            .then(_ => console.debug("Webpage hit posted."))
            .catch(error => console.log(error));
    }

    async function getHits(): Promise<number> {
        return fetch(window.location.origin+"/hits")
            .then((resp) => { 
                if (!resp.ok) throw new Error(
                    `GET /hits failed with status code ${resp.status}`
                );
                return resp.text();
            })
            .then(result => { return Number(result) })
            .catch(error => {
                console.log(error);
                return -1;
            }
        ); 
    }

    if (!hits) 
    {
        return (
            <h2 data-testid="hit-count-loading" id="hit-count">
                getting hit count...
            </h2>
        );
    } 
    else if (hits < 0) 
    {
        return (
            <h2 data-testid="hit-count-get-failed" id="hit-count">
                (unable to get visit count)
            </h2>
        );
    } 
    else 
    {
        return (
            <h2 data-testid="hit-count" id="hit-count">
                Visits so far: {hits.toLocaleString()}
            </h2>
        );
    }
}