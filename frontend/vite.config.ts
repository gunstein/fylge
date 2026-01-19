import { defineConfig } from 'vite';

export default defineConfig({
  root: '.',
  build: {
    outDir: '../static/dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: './index.html',
      },
    },
  },
  server: {
    proxy: {
      '/api': 'http://localhost:3000',
      '/markers': 'http://localhost:3000',
      '/health': 'http://localhost:3000',
      '/static/icons': 'http://localhost:3000',
    },
  },
});
