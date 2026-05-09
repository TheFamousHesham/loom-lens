import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// In dev, proxy /api and /healthz to the running Rust viewer on :7000 so
// `pnpm dev` can hot-reload the frontend without us re-bundling and
// re-running cargo build.
export default defineConfig({
  plugins: [react()],
  base: '/_loom/assets/',
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:7000',
      '/healthz': 'http://127.0.0.1:7000'
    }
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: false,
    rollupOptions: {
      output: {
        // Stable asset paths so rust-embed can serve them via /_loom/assets/.
        assetFileNames: '[name]-[hash][extname]',
        chunkFileNames: '[name]-[hash].js',
        entryFileNames: '[name]-[hash].js'
      }
    }
  }
});
