import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Конфигурация Vite, согласованная с Tauri:
// фиксированный порт 1420 нужен, потому что tauri.conf.json ссылается на него
// как на devUrl. HMR работает на отдельном порту 1421.
export default defineConfig(async () => ({
  plugins: [react()],

  // Tauri ожидает фиксированный порт и не должен очищать экран,
  // чтобы не затирать вывод Rust-компилятора.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    // Явный IPv4: на Windows `localhost` часто резолвится в ::1 (IPv6),
    // а Tauri проверяет devUrl через 127.0.0.1 → ERR_CONNECTION_REFUSED.
    host: "127.0.0.1",
    hmr: {
      protocol: "ws",
      host: "127.0.0.1",
      port: 1421,
    },
    watch: {
      // Изменения в Rust-коде отслеживает сам Tauri, не Vite.
      ignored: ["**/src-tauri/**"],
    },
  },

  // Делает переменные окружения Tauri доступными во фронтенде при необходимости.
  envPrefix: ["VITE_", "TAURI_ENV_"],

  build: {
    // Tauri поддерживает современные движки, поэтому таргет можно повысить.
    target: "es2021",
    minify: "esbuild",
    sourcemap: false,
  },
}));
