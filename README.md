# ITU Calendar

Monorepo for two ITU course calendar apps:

- `apps/extension`: TypeScript WebExtension for Chrome and Firefox.
- `apps/web`: Dioxus fullstack website scaffold for building calendars from course CRNs.

Both apps are intended to export iCalendar (`.ics`) files. The current scaffold provides the app shells and feature boundaries; OBS parsing, CRN lookup, and calendar generation are not implemented yet.

## Requirements

- Node.js 20 or newer and npm
- Rust stable with the `wasm32-unknown-unknown` target
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/guides/tools/)

## Browser extension

```sh
cd apps/extension
npm install
npm run typecheck
npm run build:chrome
npm run build:firefox
```

Load `dist/chrome` or `dist/firefox` as an unpacked extension in the corresponding browser. The content script is scoped to the ITU OBS course calendar page.

## Dioxus website

```sh
cd apps/web
dx serve --platform fullstack
```

The site starts locally with a CRN entry shell. The OBS lookup adapter and `.ics` generation module are the planned integration points for the first feature implementation.
