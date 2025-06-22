// this file allows for a stand-alone version of the functions
// defined in backend.mock.ts for vite-plugin-mock-server.
//
// should be run with Deno, in the repo root:
//
// ```
// deno --allow-env --allow-net --allow-read test-helpers/mini-lb-server.mock.ts
// ```
import { ServerResponse } from 'node:http';
import express from 'express';
import { MockHandler } from 'vite-plugin-mock-server';
import mocks from './mock-backend/backend.mock.ts';

// finding index by algorithm in case I change the order
// of the handlers later
let lbHandlerIdx = mocks.findIndex(
    (handler: MockHandler) => {
        return handler.pattern === "/lb-list-conv/conv";
    }
);

if (lbHandlerIdx < 0) throw new Error("Mock handler not found by pattern");


const lbServer = express();
lbServer.use(express.urlencoded({ extended: true }));

// req has an "any" type, because it's actually an express.Request
// type, but the query property is useable by the vite-plugin-mock-server 
// library's custom Request type
lbServer.get("/lb-list-conv/conv", (req: any, res: ServerResponse) => {

    mocks[lbHandlerIdx].handle(req, res)
});

const port: string = process.env.JS_TEST_PORT? process.env.JS_TEST_PORT: "3000";

lbServer.listen(port, () => {
  console.log(`Listening on 127.0.0.1:${port}...`);
});
