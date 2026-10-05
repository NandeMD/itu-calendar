import { copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { defineConfig, type Plugin } from "vite";
import tailwindcss from '@tailwindcss/vite'

const obsMatch = "https://obs.itu.edu.tr/ogrenci/Takvim/DersTakvimi*";
const iconSizes = [16, 32, 48, 128] as const;

function iconPaths(): Record<number, string> {
  return Object.fromEntries(iconSizes.map((size) => [size, `icons/icon-${size}.png`]));
}

function writeManifest(browser: string): Plugin {
  return {
    name: "write-webextension-manifest",
    closeBundle() {
      const outDir = resolve(__dirname, "dist", browser);
      mkdirSync(outDir, { recursive: true });

      const iconDir = resolve(outDir, "icons");
      mkdirSync(iconDir, { recursive: true });
      for (const size of iconSizes) {
        const filename = `icon-${size}.png`;
        copyFileSync(resolve(__dirname, "assets", "icons", filename), resolve(iconDir, filename));
      }

      const manifest = {
        manifest_version: 3,
        name: "ITU Calendar",
        version: "0.1.0",
        description: "Prepare calendar exports from ITU OBS course schedules.",
        icons: iconPaths(),
        action: { default_popup: "index.html", default_icon: iconPaths() },
        permissions: ["storage"],
        content_scripts: [
          { matches: [obsMatch], js: ["observe-api.js"], run_at: "document_start", world: "MAIN" },
          { matches: [obsMatch], js: ["content.js"], run_at: "document_start", world: "ISOLATED" },
        ],
        ...(browser === "firefox"
          ? {
              browser_specific_settings: {
                gecko: {
                  id: "itu-calendar@local",
                  data_collection_permissions: { required: ["none"] },
                },
              },
            }
          : {}),
      };

      writeFileSync(resolve(outDir, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
    },
  };
}

export default defineConfig(({ mode }) => {
  if (mode !== "chrome" && mode !== "firefox") {
    throw new Error(`Unsupported browser build mode: ${mode}`);
  }

  return {
    publicDir: false,
    build: {
      outDir: `dist/${mode}`,
      emptyOutDir: true,
      rollupOptions: {
        input: {
          index: resolve(__dirname, "index.html"),
          content: resolve(__dirname, "src/content/main.ts"),
          "observe-api": resolve(__dirname, "src/content/observe-api.ts"),
        },
        output: {
          entryFileNames: "[name].js",
          chunkFileNames: "chunks/[name].js",
          assetFileNames: "assets/[name][extname]",
        },
      },
    },
    plugins: [writeManifest(mode), tailwindcss()],
  };
});
