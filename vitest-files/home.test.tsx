import { describe, expect, vi, it } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { userEvent } from '@testing-library/user-event';
import HitCounter from '../static/scripts/home/hit-counter';
import ReactLogoMessage from '../static/scripts/home/react-logo-msg';

const fetchSpy = vi.spyOn(global, 'fetch')
    .mockImplementation((_a?, _b?) => {
        return Promise.resolve(
            new Response("8002934")
        )
    }
);

describe('Testing home loading hit count', () => {

    it('displays it is gonna get hit count', () => {

        render(<HitCounter />);

        expect(screen.getByRole("heading"))
            .toHaveTextContent("getting hit count...");
    });

    it('displays hit count', async () => {

        // let it load fully
        await waitFor(() => {
            expect(fetchSpy).toHaveBeenCalledTimes(2);
        });

        expect(screen.getByRole("heading"))
            .toHaveTextContent("8,002,934");
    });
});


describe('Testing the React logo flair', () => {
    
    render(<ReactLogoMessage />);
    const user = userEvent.setup();

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