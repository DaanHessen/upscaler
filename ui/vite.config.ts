import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  build: {
    outDir: '../frontend',
    emptyOutDir: true,
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/health': 'http://127.0.0.1:3000',
      '/moderate': 'http://127.0.0.1:3000',
      '/upscale': 'http://127.0.0.1:3000',
      '/upscales': 'http://127.0.0.1:3000',
      '/history': 'http://127.0.0.1:3000',
      '/balance': 'http://127.0.0.1:3000',
      '/checkout': 'http://127.0.0.1:3000',
      '/admin': 'http://127.0.0.1:3000',
    }
  }
});
