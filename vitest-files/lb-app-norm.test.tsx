/// <reference types="@vitest/browser/context" />
import { readFileSync } from 'node:fs';
import { beforeAll, beforeEach, afterEach, describe, expect, vi, it, vitest } from 'vitest';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import { userEvent } from '@testing-library/user-event';
import * as LbAppModule from '../static/scripts/lb-app/lb-app-react';

const getListSpy  = vi.spyOn(LbAppModule.testHandle, 'getListCalled'); 
const genUrlSpy   = vi.spyOn(LbAppModule.testHandle, 'getGenURL');
const startGetSpy = vi.spyOn(LbAppModule.testHandle, 'gettingList');  // after successful getList call, before first event
const eventSpy    = vi.spyOn(LbAppModule.testHandle, 'getEvent');
const completeSpy = vi.spyOn(LbAppModule.testHandle, 'isComplete');


const user = userEvent.setup();

// workaround to make the the SVG elements in the loading bar
// work with JSDOM virtual rendering (for which SVGs are not
// fully implemented)
//
// See this GitHub issue: https://github.com/DomParfitt/graphviz-react/issues/83
// (This is for another module, but it's the same fundamental issue)
beforeAll(() => {
  SVGElement.prototype.getTotalLength = vitest.fn();
});
beforeEach(() => render(<LbAppModule.LetterboxdApp />));
afterEach(()  => cleanup());

const alertSpy = vi.spyOn(global, 'alert')
    .mockImplementation((alertText: string) => {
        console.error(alertText);
        return alertText;
    }
);


describe('Ensuring all links are live', () => {

    it('has only live links', () => {
        
        // HTML without the React components (no links are in components)
        const baseHTML  = readFileSync("./index.html", { encoding: "utf8" });
        const docParser = new DOMParser();
        const baseDoc   = docParser.parseFromString(baseHTML, "text/html"); 
        const links: HTMLCollectionOf<HTMLAnchorElement> = baseDoc.getElementsByTagName("a");

        var fetchPromises = new Array();
        for (var link of links) {

            var fetchPromise = fetch(link.href).then(resp => expect(resp.ok));
            fetchPromises.push(fetchPromise);
        }

        Promise.all(fetchPromises);
    })
});

describe("Testing URL query generation", () => {

    // the label of the checkboxes, not the ID
    // unsorted, to check sorting in code
    const testCheckboxes = [
        "avg rating",
        "director",
        "makeup",
        "cast list",
    ]

    it("assembles valid URL for null attribute list", () => {

            // wait until button comes back
            return screen.findByRole("button")
                .then((_) => user.click(screen.getByTestId("url-input")))
                .then((_) => user.type(screen.getByTestId("url-input"), "https://letterboxd.com/user_exists/list/normal-one/"))
                .then((_) => user.click(screen.getByRole("button")))
                .then((_) => {
                    const correctURL = window.location.origin 
                        + "/lb-list-conv/conv?"
                        + "list_name=normal-one&author_user=user_exists"
                        + "&attrs=none";
                
                    expect(genUrlSpy).toHaveReturnedWith(correctURL);
                });
        }
    );

    
    it("assembles appropiate URL for request", () => {

        // wait until button comes back
        // requires at least 5*800ms (given the sleeps I put in 
        // the mock backend)
        return screen.findByRole("button")
            .then((_) => {

                var clickPromises = new Array();
                for (var i=0; i < testCheckboxes.length; i++) {
                    var box = screen.getByLabelText(testCheckboxes[i]);
                    clickPromises.push(user.click(box))
                }

                // single promise that only calls callback when all elements
                // are fulfilled
                return Promise.all(clickPromises);
            })
            .then((_) => user.click(screen.getByTestId("url-input")))
            .then((_) => user.clear(screen.getByTestId("url-input")))
            .then((_) => user.type(screen.getByTestId("url-input"), "https://letterboxd.com/user_exists/list/normal-one/"))
            .then((_) => user.click(screen.getByRole("button")))
            .then((_) => expect(genUrlSpy).toHaveBeenCalledTimes(2))
            .then((_) => {
                // this exact string is expected, because the attributes
                // are sorted as a part of the process
                const correctURL = window.location.origin 
                    + "/lb-list-conv/conv?"
                    + "list_name=normal-one&author_user=user_exists"
                    + "&attrs=avg-rating&attrs=cast-list&attrs=director&attrs=makeup";

                expect(genUrlSpy).toHaveReturnedWith(correctURL);
            });
    });
});


describe("Testing user interaction", () => {

    describe("pre-submission checks", () => {

        it("rejects an absent URL", () => {

            // empty URL should have been been caught prior, due to pattern attribute
            // in the HTML (via JSX) element. So getList (the form hanlder) should not 
            // have been called (it would get turned away with a 400 error anyway, but 
            // best to guard, right?)
            return screen.findByRole("button")
                .then((_) => user.clear(screen.getByTestId("url-input")))
                .then((_) => getListSpy.mockClear())
                .then((_) => user.click(screen.getByRole("button")))
                .then((_) => expect(getListSpy).not.toHaveBeenCalled());
        });
            

        it("rejects a non-URL", () => {

            // the erroneous pattern is caught in the HTML definition 
            // itself (using the `pattern` attribute of the actual <input> 
            // element), so getList, and thus parseURL should not have 
            // been called
            return screen.findByRole("button", {}, {timeout: 4500})
                .then((_) => user.click(screen.getByTestId("url-input")))
                .then((_) => user.clear(screen.getByTestId("url-input")))
                .then((_) => user.type(screen.getByTestId("url-input"), "a string that isn't a url"))
                .then((_) => getListSpy.mockClear())
                .then((_) => user.click(screen.getByRole("button")))
                .then((_) => expect(getListSpy).not.toHaveBeenCalled());
        });


        it("rejects non-Letterboxd URL", () => {

            // the erroneous pattern is caught in the HTML definition 
            // itself (using the `pattern` attribute of the actual <input> 
            // element), so getList, and thus parseURL should not have 
            // been called
            return screen.findByRole("button", {}, {timeout: 4500})
                .then((_) => user.click(screen.getByTestId("url-input")))
                .then((_) => user.clear(screen.getByTestId("url-input")))
                .then((_) => user.type(screen.getByTestId("url-input"), "https://not-letterboxd.com/which/has/an/api"))
                .then((_) => getListSpy.mockClear())
                .then((_) => user.click(screen.getByRole("button")))
                .then((_) => expect(getListSpy).not.toHaveBeenCalled());
        });


        it("rejects malformed Letterboxd URL", () => {

            // the erroneous pattern is caught in the HTML definition 
            // itself (using the `pattern` attribute of the actual <input> 
            // element), so getList, and thus parseURL should not have 
            // been called
            return screen.findByRole("button", {}, {timeout: 4500})
                .then((_) => user.click(screen.getByTestId("url-input")))
                .then((_) => user.clear(screen.getByTestId("url-input")))
                .then((_) => user.type(screen.getByTestId("url-input"), "https://letterboxd.com/but/not/an/lb-list"))
                .then((_) => getListSpy.mockClear())
                .then((_) => user.click(screen.getByRole("button")))
                .then((_) => expect(getListSpy).not.toHaveBeenCalled());
        });
    });


    describe("spinner / loading bar", () => {

        // send out new request for list
        beforeEach(() => {
            return screen.findByRole("button")
                .then((_) => user.click(screen.getByTestId("url-input")))
                .then((_) => user.clear(screen.getByTestId("url-input")))
                .then((_) => user.type(screen.getByTestId("url-input"), "https://letterboxd.com/user_exists/list/normal-one/"))
                .then((_) => user.click(screen.getByRole("button")));
        });

        it("hides Get My List! button when getting list", () => {

            expect(screen.queryByTestId("submit-button")).not.toBeInTheDocument();
        });


        it("shows spinner before first received event", () => {

            // wait getList started successfully, but before first event
            return waitFor(() => expect(startGetSpy).toReturnWith(true))
                .then((_) => {
                    let ringLoader = screen.getByTestId("ring-loader");
                    expect(ringLoader).toBeInTheDocument()
                });
        });


        it("shows loading bar with percentage after first received event", () => {

            // wait until after first event is received
            return waitFor(() => expect(eventSpy).toHaveBeenCalled())
                .then((_) => screen.findByTestId("loading-bar"))
                .then((_) => screen.findByText("%", {exact: false}))
                .then((percentage) => expect(percentage).toBeInTheDocument());
        });


        it("shows Get My List! button again after list is received in full", {timeout: 6000}, () => {

            // wait until after the "done!" signal has been received
            return waitFor(() => expect(completeSpy).toHaveBeenCalled())
                .then((_) => screen.findByRole("button", {}, {timeout: 6000}))
                .then((button) => expect(button).toBeInTheDocument())
        });
    });
});