/// <reference types="@vitest/browser/context" />

// these tests were separated out because it was becoming too much
// to orchestrate multiple async tests on the same DOM
import { describe, expect, vi, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { userEvent } from '@testing-library/user-event';
import * as LbAppModule from '../static/scripts/lb-app/lb-app-react';

const user = userEvent.setup();
render(<LbAppModule.LetterboxdApp />);

window.alert = vi.fn((alertText: string) => {
    console.error(alertText);
    return alertText;
});

const dlSpy = vi.spyOn(LbAppModule.testHandle, 'dlListCalled');


describe("Testing error states", () => {

    // When an in-stream error occurs, the component should restore 
    // the submit button and not show the loading animations

    it("informs user that the list is too long", async () => {
        await screen.findByRole("button", {}, {timeout: 4500})
            .then(async () => {
                await user.click(screen.getByTestId("url-input"));
                await user.clear(screen.getByTestId("url-input"));
                await user.type(screen.getByTestId("url-input"), "https://letterboxd.com/user_exists/list/list-too-long/");
                await user.click(screen.getByRole("button"));
            });
        
        // wait for alert to pop up
        setTimeout(() => {
            
            expect(window.alert).toReturnWith(
                "The server does not accept conversion requests for \
                lists over 10,000 films long. Is there a shorter list we can try?"
                .replaceAll("  ", "")
            );
            
            expect(screen.getByRole("button")).toBeInTheDocument();
        }, 100);

        expect(dlSpy).not.toHaveBeenCalled();  // make sure a download wasn't triggered
    });


    it("informs user that the list is invalid", async () => {

        await screen.findByRole("button", {}, {timeout: 4500})
            .then(async () => {
                await user.click(screen.getByTestId("url-input"));
                await user.clear(screen.getByTestId("url-input"));
                await user.type(screen.getByTestId("url-input"), "https://letterboxd.com/user_exists/list/list-no-exist/");
                await user.click(screen.getByRole("button"));
            });

        // wait for alert to pop up
        setTimeout(() => {
            expect(window.alert).toReturnWith(
                "The URL entered doesn't appear to be a valid Letterboxd list. \
                Try checking the link and running it again."
                .replaceAll("  ", "")
            );

            expect(screen.getByRole("button")).toBeInTheDocument();
        }, 100);

        expect(dlSpy).not.toHaveBeenCalled();  // make sure a download wasn't triggered
    });


    it("informs user that the Letterboxd servers are down", async () => {

        await screen.findByRole("button", {}, {timeout: 4500})
            .then(async () => {
                await user.click(screen.getByTestId("url-input"));
                await user.clear(screen.getByTestId("url-input"));
                await user.type(screen.getByTestId("url-input"), "https://letterboxd.com/user_exists/list/lb-server-down/");
                await user.click(screen.getByRole("button"));
            });

        // wait for alert to pop up
        setTimeout(() => {
            expect(window.alert).toReturnWith(
                "It looks like Letterboxd's servers are down! Try again a little later."
            );

            expect(screen.getByRole("button")).toBeInTheDocument();
        }, 100);

        expect(dlSpy).not.toHaveBeenCalled();  // make sure a download wasn't triggered
    });

    
    it("informs user that Archie is down", async () => {

        await screen.findByRole("button", {}, {timeout: 4500})
            .then(async () => {
                await user.click(screen.getByTestId("url-input"));
                await user.clear(screen.getByTestId("url-input"));
                await user.type(screen.getByTestId("url-input"), "https://letterboxd.com/user_exists/list/this-hurts-you/");
                await user.click(screen.getByRole("button"));
            });

        expect(window.alert).toReturnWith(
            "There was an issue with the server itself that prevented the completion \
            of your request. My apologies.".replaceAll("  ", "")
        );

        expect(screen.getByRole("button")).toBeInTheDocument();

        expect(dlSpy).not.toHaveBeenCalled();  // make sure a download wasn't triggered
    });
});