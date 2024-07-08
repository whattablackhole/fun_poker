import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react-swc";

export default defineConfig(({ mode }) => {
  process.env = { ...process.env, ...loadEnv(mode, process.cwd()) };

  const run_in_docker = !!process.env.VITE_RUN_IN_DOCKER;

  const api_url_target = process.env.VITE_API_URL;

  if (run_in_docker) {
    process.env.VITE_API_URL = "/api";
  }

  return {
    plugins: [react()],
    server: {
      https: {
        cert: `${process.env.VITE_CERT_PATH}`,
        key: `${process.env.VITE_PRIVATE_KEY_PATH}`,
      },
      watch: {
        usePolling: run_in_docker,
      },
      proxy:
        mode === "development" && run_in_docker
          ? {
              "/api": {
                target: api_url_target,
                changeOrigin: true,
                rewrite: (path) => path.replace(/^\/api/, ""),
                secure: false,
              },
            }
          : undefined,
    },
  };
});
