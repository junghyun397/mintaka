import {defineConfig} from 'vite';
import solidPlugin from 'vite-plugin-solid';
import devtools from "solid-devtools/vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [
    solidPlugin(),
    devtools(),
    tailwindcss()
  ],
  server: {
    port: 3000,
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
  preview: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
  build: {
    target: 'esnext',
  },
});
