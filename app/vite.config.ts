import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [sveltekit()],
  // Tauri expects a fixed port and shows Rust errors itself.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    // Translations live in ../locales, shared with the Rust list renderer.
    fs: { allow: ['..'] },
    watch: { ignored: ['**/src-tauri/**'] }
  },
  test: {
    include: ['src/**/*.test.ts']
  }
});
