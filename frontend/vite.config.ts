import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

const API_TARGET = process.env.VITE_API_URL ?? "http://localhost:3001";

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    host: "0.0.0.0",
    port: 3000,
    proxy: {
      "/api": {
        target: API_TARGET,
        changeOrigin: true,
        secure: false,
      },
    },
  },
});
