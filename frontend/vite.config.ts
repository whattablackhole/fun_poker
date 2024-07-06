import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react-swc";

export default defineConfig(({ mode }) => {
  process.env = { ...process.env, ...loadEnv(mode, process.cwd()) };

  return {
    plugins: [react()],
    server: {
      https: {
        cert: `${process.env.VITE_CERT_PATH}`,
        key: `${process.env.VITE_PRIVATE_KEY_PATH}`,
      },
    },
  };
});
