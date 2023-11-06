import {
  defineConfig,
  loadEnv,
  type Plugin,
  type HtmlTagDescriptor,
} from "vite";
import react from "@vitejs/plugin-react";
// import legacy from "@vitejs/plugin-legacy";

const csp: Record<string, string> = {
  "default-src": "'self'",
  "script-src":
    "'self' spec: figapp: blob: https://d3e7ef0le33nq1.cloudfront.net http://localhost:*",
  "img-src": "'self' data: fig: figapp: https: http://localhost:*",
  "style-src":
    " 'self' 'unsafe-inline' figapp: https://d3e7ef0le33nq1.cloudfront.net http://localhost:*",
  "connect-src": "'self' ws: http://localhost:*",
  "object-src": "'none'",
  "frame-src": "'none'",
};

const cspContent = Object.entries(csp)
  .map(([k, v]) => `${k} ${v}`)
  .join("; ");

const htmlCspPlugin: Plugin = {
  name: "html-csp",
  transformIndexHtml: {
    order: "post",
    handler: (_html, ctx): HtmlTagDescriptor[] => {
      if (ctx.server?.config?.mode === "development") {
        return [];
      }

      return [
        {
          injectTo: "head",
          tag: "meta",
          attrs: {
            "http-equiv": "Content-Security-Policy",
            content: cspContent,
          },
        },
      ];
    },
  },
};

// https://vitejs.dev/config/
export default defineConfig(({ mode, command }) => {
  process.env = { ...process.env, ...loadEnv(mode, process.cwd(), "") };

  return {
    plugins: [
      react(),
      htmlCspPlugin,
      // legacy({
      //   targets: [
      //     "safari >= 11",
      //     "last 3 Chrome version",
      //     "last 3 Firefox version",
      //   ],
      // }),
    ],
    css: {
      modules: {
        localsConvention: "camelCaseOnly",
      },
    },
    server: {
      port: process.env.PORT ? parseInt(process.env.PORT, 10) : 3124,
      strictPort: true,
    },
    build: {
      target: command === "build" ? "es2017" : "esnext",
      // TODO: re-enable prod source maps to upload them to sentry (see build CIs)
      sourcemap: command !== "build",
      rollupOptions: {
        external: [
          "?type=option",
          "?type=carrot",
          "?type=command",
          "?type=box",
        ],
      },
    },
    define: {
      __APP_VERSION__: JSON.stringify(process.env.npm_package_version),
      "process.env": {},
    },
    esbuild: {
      target: command === "build" ? ["es2017", "safari11"] : undefined,
    },
    resolve: {
      alias: {
        util: "util",
      },
    },
  };
});
