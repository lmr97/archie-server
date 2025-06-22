/// <reference types="@vitest/browser/context" />
import { describe, expect, vi, it } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { userEvent } from '@testing-library/user-event';
import * as LbAppModule from '../static/scripts/lb-app/lb-app-react';

const getListSpy  = vi.spyOn(LbAppModule.testHandle, 'getListCalled'); 
const genUrlSpy   = vi.spyOn(LbAppModule.testHandle, 'getGenURL');
const startGetSpy = vi.spyOn(LbAppModule.testHandle, 'gettingList');  // after successful getList call, before first event
const eventSpy    = vi.spyOn(LbAppModule.testHandle, 'getEvent');
const completeSpy = vi.spyOn(LbAppModule.testHandle, 'isComplete');


const user = userEvent.setup();
render(<LbAppModule.LetterboxdApp />);

const alertSpy = vi.spyOn(global, 'alert')
    .mockImplementation((alertText: string) => {
        console.error(alertText);
        return alertText;
    }
);

describe("Testing URL query generation", () => {

    // the label of the checkboxes, not the ID
    // unsorted, to check sorting in code
    const testCheckboxes = [
        "avg rating",
        "director",
        "makeup",
        "cast list",
    ]

    it("assembles valid URL for null attribute list", async () => {

        // wait until button comes back
        await screen.findByRole("button")
            .then(async (_) => {

                // click URL input box
                await user.click(screen.getByTestId("url-input"));
                await user.type(
                    screen.getByTestId("url-input"), 
                    "https://letterboxd.com/user_exists/list/normal-one/"
                );
                await user.click(screen.getByRole("button"));
            });

        const correctURL = window.location.origin 
            + "/lb-list-conv/conv?"
            + "list_name=normal-one&author_user=user_exists"
            + "&attrs=none";
    
        expect(genUrlSpy).toHaveReturnedWith(correctURL);
    });

    
    it("assembles appropiate URL for request", {timeout: 10000}, async () => {

        // wait until button comes back
        // requires at least 5*800ms (given the sleeps I put in 
        // the mock backend)
        await screen.findByRole("button", {}, {timeout: 5000})
            .then(async (_) => {

                // URL already in input box

                for (var i=0; i < testCheckboxes.length; i++) {
                    var box = screen.getByLabelText(testCheckboxes[i]);
                    await user.click(box);
                }
                await user.click(screen.getByRole("button"));
            });
        
        // wait for URL generator to be called
        await waitFor(() => {
            expect(genUrlSpy).toHaveBeenCalledTimes(2);
        })

        // this exact string is expected, because the attributes
        // are sorted as a part of the process
        const correctURL = window.location.origin 
            + "/lb-list-conv/conv?"
            + "list_name=normal-one&author_user=user_exists"
            + "&attrs=avg-rating&attrs=cast-list&attrs=director&attrs=makeup";

        expect(genUrlSpy).toHaveReturnedWith(correctURL);
    });
});


describe("Testing user interaction", () => {

    describe("pre-submission checks", () => {

        it("rejects an absent URL", async () => {

            // wait until button comes back
            await screen.findByRole("button", {}, {timeout: 4500})
                .then(async () => {
                    await user.clear(screen.getByTestId("url-input"));
                    getListSpy.mockClear();
                    await user.click(screen.getByRole("button"));
                });
            
            // empty URL should have been been caught prior, due to pattern attribute
            // in the HTML (via JSX) element. So getList (the form hanlder) should not 
            // have been called (it would get turned away with a 400 error anyway, but 
            // best to guard, right?)
            expect(getListSpy).not.toHaveBeenCalled();
        });
            

        it("rejects a non-URL", async () => {

            // wait for submit button to come back
            await screen.findByRole("button", {}, {timeout: 4500})
                .then(async () => {
                    await user.click(screen.getByTestId("url-input"));
                    await user.clear(screen.getByTestId("url-input"));
                    await user.type(screen.getByTestId("url-input"), "a string that isn't a url");
                    getListSpy.mockClear();
                    await user.click(screen.getByRole("button"));
                });
            
            // the erroneous pattern is caught in the HTML definition 
            // itself (using the `pattern` attribute of the actual <input> 
            // element), so getList, and thus parseURL should not have 
            // been called
            expect(getListSpy).not.toHaveBeenCalled();
        });


        it("rejects non-Letterboxd URL", async () => {

            // wait for submit button to come back
            await screen.findByRole("button", {}, {timeout: 4500})
                .then(async () => {
                    await user.click(screen.getByTestId("url-input"));
                    await user.clear(screen.getByTestId("url-input"));
                    await user.type(screen.getByTestId("url-input"), "https://not-letterboxd.com/which/has/an/api");
                    getListSpy.mockClear();
                    await user.click(screen.getByRole("button"));
                });
            
            // the erroneous pattern is caught in the HTML definition 
            // itself (using the `pattern` attribute of the actual <input> 
            // element), so getList, and thus parseURL should not have 
            // been called
            expect(getListSpy).not.toHaveBeenCalled();
        });


        it("rejects malformed Letterboxd URL", async () => {

            // wait for submit button to come back
            await screen.findByRole("button", {}, {timeout: 4500})
                .then(async () => {
                    await user.click(screen.getByTestId("url-input"));
                    await user.clear(screen.getByTestId("url-input"));
                    await user.type(screen.getByTestId("url-input"), "https://letterboxd.com/but/not/an/lb-list");
                    getListSpy.mockClear();
                    await user.click(screen.getByRole("button"));
                });

            // the erroneous pattern is caught in the HTML definition 
            // itself (using the `pattern` attribute of the actual <input> 
            // element), so getList, and thus parseURL should not have 
            // been called
            expect(getListSpy).not.toHaveBeenCalled();
        });
    });


    describe("spinner / loading bar", () => {

        it("hides Get My List! button when getting list", async () => {

            await screen.findByRole("button")
                .then(async () => {
                    await user.click(screen.getByTestId("url-input"));
                    await user.clear(screen.getByTestId("url-input"));
                    await user.type(screen.getByTestId("url-input"), "https://letterboxd.com/user_exists/list/normal-one/");
                    await user.click(screen.getByRole("button"));
                });
                
            // queryByTestId returns null if the element is not in the document
            // give the document a moment to hide the button
            setTimeout(() => {
                expect(screen.queryByTestId("submit-button")).not.toBeInTheDocument();
            }, 300);
        });


        it("shows spinner before first received event", async () => {

            // wait getList started successfully, but before first event
            await waitFor(() => {
                expect(startGetSpy).toReturnWith(true);
            })
            
            // give the document a moment to show the ring loader
            setTimeout(() => {
                expect(screen.getByTestId("ring-loader")).toBeInTheDocument();
            }, 300);
            
            
        });


        it("shows loading bar with percentage after first received event", async () => {

            // wait until after first event is received
            await waitFor(() => {
                expect(eventSpy).toHaveBeenCalled()
            });

            setTimeout(() => {
                expect(screen.getByTestId("loading-bar")).toBeInTheDocument();
                expect(screen.getByText("%")).toBeInTheDocument();
            }, 300);
        });


        it("shows Get My List! button again after list is received in full", async () => {

            // wait until after the "done!" signal has been received
            await waitFor(() => {
                expect(completeSpy).toHaveBeenCalled();
            });
            
            setTimeout(() => {
                expect(screen.getByRole("button")).toBeInTheDocument()
            }, 300);

        });
    });

});