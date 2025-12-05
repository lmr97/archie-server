/// <reference types="@vitest/browser/context" />
import { readFileSync } from 'node:fs';
import { beforeAll, describe, expect, vi, it, afterAll } from 'vitest';
import { render, screen, cleanup, waitFor } from '@testing-library/react';
import { userEvent } from '@testing-library/user-event';
import HitCounter from '../static/scripts/home/hit-counter';
import ReactLogoMessage from '../static/scripts/home/react-logo-msg';

 
const fetchSpy = vi.spyOn(global, 'fetch')
    .mockImplementation((_a?, _b?) => {
        if (globalThis.serverFail) {
            return Promise.resolve(
                new Response(null, {status: 500})
            );
        }
        return Promise.resolve(
            new Response("8002934")
        )
    }
);

globalThis.serverFail = false;


describe('Ensuring all links are live', () => {

    it('has only live links', async () => {
        
        // HTML without the React components (no links are in components)
        const baseHTML  = readFileSync("./index.html", { encoding: "utf8" });
        const docParser = new DOMParser();
        const baseDoc   = docParser.parseFromString(baseHTML, "text/html"); 
        const links: HTMLCollectionOf<HTMLAnchorElement> = baseDoc.getElementsByTagName("a");
  
        for (var link of links) {

            var resp = await fetch(link.href);
            expect(resp.ok);
        }
    })
})

describe('Testing home loading hit count', () => {

    beforeAll(() => render(<HitCounter />));
    afterAll(()  => cleanup())
    
    it('displays it is gonna get hit count', () => {
        expect(screen.getByRole("heading")).toHaveTextContent("getting hit count...");
    });

    
    it('displays hit count', () => {
        return waitFor(() => 
            expect(screen.getByRole("heading")).toHaveTextContent("8,002,934")
        ).then((_) => fetchSpy.mockClear());
    });
});


describe('Testing home loading hit count, but server fails', () => {

    beforeAll(() => {
        globalThis.serverFail = true;
        render(<HitCounter />)
    });
    afterAll(()  => {
        cleanup()
        globalThis.serverFail = false;
    });
    
    it('displays it is gonna get hit count', () => {

        expect(screen.getByRole("heading"))
            .toHaveTextContent("getting hit count...");
    });

    
    it('tells user it could not get hit count', () => {
        return waitFor(() => 
            expect(screen.getByRole("heading")).toHaveTextContent("(unable to get visit count)")
        );
    });
});


describe('Testing the React logo flair', () => {

    
    const user = userEvent.setup();
    beforeAll(() => render(<ReactLogoMessage />));
    afterAll(()  => cleanup())


    it('does not show message when mouse is away', () => {
        
        expect(screen.getByRole("paragraph"))
            .not.toBeVisible()
    })


    it('does show message when moused over', () => {

        user.hover(screen.getByTestId("react-logo-div"));

        // wait for the animation to complete
        setTimeout(()=>{
            expect(screen.getByRole("paragraph")).toBeVisible()
        },
        1200);
    });


    it('enlarges React logo', () => {

        user.hover(screen.getByTestId("react-logo-div"));

        // wait for the animation to complete
        setTimeout(()=>{
            expect(screen.getByTestId("active-react-logo"))
                .toHaveAttribute("viewBox", "-13.5 -13.5 27 27");
        },
        1200);
    })
})