import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

const server = process.env.ASTRUM_URL ?? 'http://127.0.0.1:8081';
const proxy = Object.fromEntries(
  ['/service', '/repository', '/v2'].map((path) => [path, { target: server }]),
);

export default defineConfig({
  plugins: [react()],
  server: {
    port: Number(process.env.PORT ?? 5173),
    strictPort: true,
    proxy,
  },
  preview: { proxy },
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    sourcemap: false,
    chunkSizeWarningLimit: 900,
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}', 'scripts/**/*.test.mjs'],
    css: false,
    restoreMocks: true,
  },
});
