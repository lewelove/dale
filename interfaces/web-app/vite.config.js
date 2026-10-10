import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import path from 'path'
import net from 'node:net'
import tls from 'node:tls'
import { Stream } from 'node:stream'
import http from 'node:http'

const patch = function () { this.destroy(); };
[net.Socket, tls.TLSSocket, Stream].forEach((cls) => {
  if (cls && cls.prototype && !cls.prototype.destroySoon) {
    cls.prototype.destroySoon = patch;
  }
});

const targetUrl = process.env.DALE_SERVER_URL || 'http://127.0.0.1:8000';
const wsTargetUrl = process.env.DALE_WS_URL || 'ws://127.0.0.1:8000';;

const proxyConfig = {
  '/api': {
    target: targetUrl,
    changeOrigin: true,
    secure: false,
    agent: new http.Agent({ keepAlive: true }),
  },
  '/ws': {
    target: wsTargetUrl,
    ws: true
  }
};

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      '$modules': path.resolve(import.meta.dirname, './src/modules'),
    }
  },
  server: {
    port: 5173,
    proxy: proxyConfig,
  },
  preview: {
    port: 4173,
    proxy: proxyConfig,
  },
})
