/// <reference types="vitest/config" />
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react';
import mockServer from 'vite-plugin-mock-server'
import bodyParser from 'body-parser';
import 'jsdom';

// notes:
//
// - `expect` from Vitest needs to be extended by `jest-dom`
//   in order to get the DOM assertion methods (see vitest-setup.ts)
// - if some modules are blocked from loading, try turning of UBlockOrigin

const env = process.env
const __dirname = dirname(fileURLToPath(import.meta.url))

// for localhost, port 3000 (the Node default) is used,
// and I cannot find a way to override this. Thankfully, 
// I can specify the port for the dev server
export default defineConfig({
  test: {
    css: true,
    setupFiles: ['./vitest-files/vitest-setup.ts'],  // needs to be in "test" property
    environment: 'jsdom',
    environmentOptions: {
      url: "http://localhost:3000"
    },
    //reporters: ['hanging-process'],
    include: [ 'vitest-files/*', ],
    includeSource: [ 'static/scripts/**' ],
    coverage: {
      provider: 'v8',
      include: [ 'static/scripts/**' ],
      exclude: [ 'static/scripts/guestbook.js' ]
    }
  },
  plugins: [
    react(),
    mockServer({
      logLevel: 'warn',
      urlPrefixes: [ 
        '/hits', 
        '/guestbook',
        '/guestbook/entries',
        '/lb-list-conv',
        '/lb-list-conv/conv',
      ],
      mockRootDir: './test-helpers/mock-backend',
      mockJsSuffix: '.mock.js',
      mockTsSuffix: '.mock.ts',
      noHandlerResponse404: false,
      middlewares: [
        bodyParser.json(),
      ],
      printStartupLog: false
    })
  ],
  server: {
    cors: {
      // the origin you will be accessing via browser
      origin: `${env.SERVER_PROTOCOL}://${env.SERVER_SOCKET}`
    },
  },
  build: {
    manifest: true,
    outDir: '/home/martinr/archie-server/dist',
    rollupOptions: {
      input: [
        resolve(__dirname, 'index.html'),
        resolve(__dirname, 'pages/guestbook.html'),
        resolve(__dirname, 'pages/lb-list-app.html'),
        resolve(__dirname, 'static/errors/403.html'),
        resolve(__dirname, 'static/errors/404.html'),
        resolve(__dirname, 'static/errors/500.html'),
      ],
    },
  },
})