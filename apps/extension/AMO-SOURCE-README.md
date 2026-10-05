# ITU Calendar Firefox source submission

This archive contains the source used to build the Firefox extension submitted
to addons.mozilla.org (AMO). The Firefox package is generated from the
TypeScript and CSS files in `src/` using Vite and Tailwind CSS.

## Build environment

The submitted package was built on Linux x86_64 (CachyOS, kernel
7.2.9-1-cachyos) with Node.js 22.23.3 and npm 12.2.0.

## Reproduce the Firefox package

From this directory, run:

```sh
npm ci
npm run build:firefox
```

The generated extension files are written to `dist/firefox/`. The AMO upload
ZIP is made from the contents of that directory, with `manifest.json` at the
ZIP root. `package-lock.json` pins the npm dependency tree used by the build.

The build tools and runtime dependencies are installed from the npm registry
by `npm ci`; no private dependencies or generated source files are required.
