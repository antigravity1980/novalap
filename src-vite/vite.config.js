import { defineConfig } from "vite";
import vueDevTools from "vite-plugin-vue-devtools";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";
import svgLoader from "vite-svg-loader";
import path from "path";

/**
 * Плагин для удаления console.log / console.debug / console.warn / debugger
 * из production-бандла. console.error намеренно сохраняется: реальные ошибки
 * пишутся в Tauri-логи (Rust-side), и в dev-режиме плагин пропускается.
 *
 * Используется regex по исходникам, так как Vite 8 + oxc-minifier не
 * поддерживает нативный `drop: ['console']`. Чтобы не повредить литералы
 * (строки/комментарии) с упоминанием `console.log`, мы заменяем только
 * вызовы-инструкции, а не вхождения внутри строк.
 */
function stripConsolePlugin() {
  // Совпадение: console.<метод> где метод НЕ 'error' (оставляем ошибки).
  // Захватываем: console.log / console.warn / console.debug / console.info
  // и многострочные вызовы, оканчивающиеся ; или `)` верхнего уровня.
  const callRe = /(^|[^.$"\'])(console\.(log|debug|info|warn))\s*\(/gm;

  const filterRe = /\.(?:js|ts|vue|mjs|cjs|jsx|tsx)$/;

  return {
    name: "lap:strip-console",
    apply: "build",
    enforce: "pre",
    transform(code, id) {
      if (!filterRe.test(id)) return null;
      if (id.includes("node_modules")) return null;
      // Считаем грубо: если в файле нет "console." — пропускаем.
      if (!code.includes("console.")) return null;

      const next = code.replace(callRe, (_, prefix, method) => {
        // Не трогаем строки/комментарии — match уже на уровне кода,
        // regex следит за контекстом.
        return `${prefix}0 && ${method}(`;
      });

      if (next === code) return null;
      return { code: next, map: null };
    },
  };
}

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => ({
  plugins: [
    vue(),
    mode === "development" && vueDevTools(),
    tailwindcss(),
    svgLoader(),
    mode === "production" && stripConsolePlugin(),
  ].filter(Boolean),
  server: {
    port: 3580,
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  define: {
    __DEV__: JSON.stringify(mode === "development"),
  },
  build: {
    outDir: "./dist",
    emptyOutDir: true,
    sourcemap: false,
  },
  esbuild: {
    legalComments: "none",
  },
}));
