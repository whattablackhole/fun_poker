import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react-swc";

export default defineConfig(({ mode }) => {
  process.env = { ...process.env, ...loadEnv(mode, process.cwd()) };

  const usePolling = !!process.env.VITE_RUN_IN_DOCKER;

  return {
    plugins: [react()],
    server: {
      https: {
        cert: `${process.env.VITE_CERT_PATH}`,
        key: `${process.env.VITE_PRIVATE_KEY_PATH}`,
      },
      watch: {
        usePolling: usePolling,
      },
      proxy:
        mode === "development"
          ? {
              "/api": {
                target: process.env.VITE_API_URL,
                changeOrigin: true,
                rewrite: (path) => path.replace(/^\/api/, ""),
                secure: false,
              },
            }
          : undefined,
    },
  };
});
