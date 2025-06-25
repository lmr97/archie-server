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

    const [hits, setHits]: [number, Function] = useState(0);
    // for display purposes, we only care if the GET fails
    // POST errors are simply logged, on client and server
    const [getErrorOccured, setGetErrorOccured]: [boolean, Function] = useState(false);

    useEffect(() => {
            if (!hitPosted.current && !getErrorOccured) {

                postHit().catch(error => console.error(error));
                hitPosted.current = true;
            }
            getHits()
                .then((hitCount: number) => { 
                    if (hitCount > hits) {
                        setHits(hitCount);
                    }
                })
        },
        [hits]
    )

    async function postHit(): Promise<void | Error> {
        return fetch(window.location.origin+"/hits", postOptions)
            .then(resp => {
                if (!resp.ok) throw new Error(
                    `POST hit to /hits failed with status code ${resp.status}`
                );
            })
            .then(_ => console.debug("Webpage hit posted."))
    }

    async function getHits(): Promise<number> {
        return fetch(window.location.origin+"/hits")
            .then((resp) => { 
                if (!resp.ok) throw new Error(
                    `GET /hits failed with status code ${resp.status}`
                );
                return resp.text();
            })
            .then(hits => { return Number(hits) })
            .catch((hitGettingError) => {
                console.error(hitGettingError);
                setGetErrorOccured(true); 
                return 0;
            }); 
    }

    if (!hits && !getErrorOccured) 
    {
        return (
            <h2 data-testid="hit-count-loading" id="hit-count">
                getting hit count...
            </h2>
        );
    } 
    if (getErrorOccured) 
    {
        return (
            <h2 data-testid="hit-count-get-failed" id="hit-count">
                (unable to get visit count)
            </h2>
        );
    } 

    return (
        <h2 data-testid="hit-count" id="hit-count">
            Visits so far: {hits.toLocaleString()}
        </h2>
    );
    
}