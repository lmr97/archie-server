import { afterEach, describe, expect, vi, it } from 'vitest';
import { render, screen, waitFor, cleanup } from '@testing-library/react';
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

afterEach(() => cleanup());
globalThis.serverFail = false;


describe('Testing home loading hit count', () => {

    it('displays it is gonna get hit count', () => {
        render(<HitCounter />);

        expect(screen.getByRole("heading"))
            .toHaveTextContent("getting hit count...");
    });

    it('displays hit count', () => {

        render(<HitCounter />);
        
        // let it load fully. I understand that this is a goofy
        // way to do this, but doing it this way allows me to cut
        // out a lot of async bullshit, which I want
        setTimeout(() => {
                expect(screen.getByRole("heading"))
                    .toHaveTextContent("8,002,934");
            
                fetchSpy.mockClear();
                cleanup();
            },
            500
        );
    });
});


describe('Testing home loading hit count, but server fails', () => {

    globalThis.serverFail = true;

    it('displays it is gonna get hit count', () => {

        render(<HitCounter />);
        expect(screen.getByRole("heading"))
            .toHaveTextContent("getting hit count...");
    });

    it('tells user it could not get hit count', () => {

        render(<HitCounter />);
        
        // let it load fully
        setTimeout(() => {
                expect(screen.getByRole("heading"))
                    .toHaveTextContent("(unable to get visit count)");
            
                cleanup();
            },
            500
        );
    });
});


describe('Testing the React logo flair', () => {

    const user = userEvent.setup();

    it('does not show message when mouse is away', () => {

        render(<ReactLogoMessage />);
        expect(screen.getByRole("paragraph"))
            .not.toBeVisible()
    })

    it('does show message when moused over', () => {

        render(<ReactLogoMessage />);
        user.hover(screen.getByTestId("react-logo-div"));

        // wait for the animation to complete
        setTimeout(()=>{
            expect(screen.getByRole("paragraph")).toBeVisible()
        },
        1200);
    });

    it('enlarges React logo', () => {

        render(<ReactLogoMessage />);
        user.hover(screen.getByTestId("react-logo-div"));

        // wait for the animation to complete
        setTimeout(()=>{
            expect(screen.getByTestId("active-react-logo"))
                .toHaveAttribute("viewBox", "-13.5 -13.5 27 27");
        },
        1200);
    })
})