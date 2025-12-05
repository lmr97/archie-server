/// <reference types="@vitest/browser/context" />

import { describe, expect, vi, it, beforeEach, beforeAll, MockInstance, afterAll, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { userEvent } from '@testing-library/user-event';
import * as LbAppModule from '../static/scripts/lb-app/lb-app-react';

const user = userEvent.setup();
render(<LbAppModule.LetterboxdApp />);

const alertSpy = vi.spyOn(window, 'alert');
const dlSpy    = vi.spyOn(LbAppModule.testHandle, 'dlListCalled');

alertSpy.mockImplementation(
    vi.fn((alertText: string) => {
        console.error(alertText);
        return alertText;
    })
);

describe("Testing error states", () => {

    const urlInput = screen.getByTestId("url-input");

    beforeEach(() => {
        return screen.findByRole("button")
            .then((_) => user.click(urlInput))
            .then((_) => user.clear(urlInput))
    });

    // When an in-stream error occurs, the component should restore 
    // the submit button and not show the loading animations

    it("informs user that the list is too long", () => {
        
        return user.type(urlInput, "https://letterboxd.com/user_exists/list/list-too-long/")
            .then((_) => user.click(screen.getByRole("button")))
            .then((_) => waitFor(() => expect(alertSpy).toHaveBeenCalled()))
            .then((_) => {
                expect(alertSpy).toReturnWith(
                    "The server does not accept conversion requests for \
                    lists over 10,000 films long. Is there a shorter list we can try?"
                    .replaceAll("  ", "")
                );
                expect(screen.getByRole("button")).toBeInTheDocument();
                expect(dlSpy).not.toHaveBeenCalled();  // make sure a download wasn't triggered
            });
    });


    it("informs user that the list is invalid", () => {

        return user.type(urlInput, "https://letterboxd.com/user_exists/list/list-no-exist/")
            .then((_) => user.click(screen.getByRole("button")))
            .then((_) => waitFor(() => expect(alertSpy).toHaveBeenCalled()))
            .then((_) => {
                expect(alertSpy).toReturnWith(
                    "The URL entered doesn't appear to be a valid Letterboxd list. \
                    Try checking the link and running it again."
                    .replaceAll("  ", "")
                );

                expect(screen.getByRole("button")).toBeInTheDocument();
                expect(dlSpy).not.toHaveBeenCalled();  // make sure a download wasn't triggered
            })
    });


    it("informs user that the Letterboxd servers are down", () => {

        return user.type(urlInput, "https://letterboxd.com/user_exists/list/lb-server-down/")
            .then((_) => user.click(screen.getByRole("button")))
            .then((_) => waitFor(() => expect(alertSpy).toHaveBeenCalled()))
            .then((_) => {
                expect(alertSpy).toReturnWith(
                    "It looks like Letterboxd's servers are down! Try again a little later."
                );

                expect(screen.getByRole("button")).toBeInTheDocument();
                expect(dlSpy).not.toHaveBeenCalled();  // make sure a download wasn't triggered
            });
    });

    
    it("informs user that Archie is down", () => {

        return user.type(urlInput, "https://letterboxd.com/user_exists/list/this-hurts-you/")
            .then((_) => user.click(screen.getByRole("button")))
            .then((_) => waitFor(() => expect(alertSpy).toHaveBeenCalled()))
            .then((_) => {
                expect(alertSpy).toReturnWith(
                    "There was an issue with the server itself that prevented the completion \
                    of your request. My apologies.".replaceAll("  ", "")
                );

                expect(screen.getByRole("button")).toBeInTheDocument();
                expect(dlSpy).not.toHaveBeenCalled();  // make sure a download wasn't triggered
            });
    });
});