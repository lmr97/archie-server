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
    const [errorOccured, setErrorOccured]: [boolean, Function] = useState(false);

    useEffect(() => {
            if (!hitPosted.current && !errorOccured) {

                postHit().catch(error => {
                    console.error(error);
                    setErrorOccured(true);   // triggers a refresh, 
                });
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
                setErrorOccured(true);
                return 0;
            }); 
    }

    if (!hits && !errorOccured) 
    {
        return (
            <h2 data-testid="hit-count-loading" id="hit-count">
                getting hit count...
            </h2>
        );
    } 
    if (errorOccured) 
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