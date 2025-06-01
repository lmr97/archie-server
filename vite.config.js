import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react';
import mockServer from 'vite-plugin-mock-server'
import bodyParser from 'body-parser';

const env = process.env
const __dirname = dirname(fileURLToPath(import.meta.url))

export default defineConfig({
  plugins: [
    react(),
    mockServer({
      logLevel: 'warn',
      urlPrefixes: [ '/hits', '/guestbook/entries', '/lb-list-conv' ],
      mockRootDir: './mock-backend',
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
    rollupOptions: {
      input: {
        home:      resolve(__dirname, 'index.html'),
        guestbook: resolve(__dirname, 'pages/guestbook.html'),
        lb_app:    resolve(__dirname, 'pages/lb-list-app.html'),
        error403:  resolve(__dirname, 'static/errors/403.html'),
        error404:  resolve(__dirname, 'static/errors/404.html'),
        error500:  resolve(__dirname, 'static/errors/500.html'),
      },
    },
  },
})