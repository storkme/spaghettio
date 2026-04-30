import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // happy-dom gives us `window`, `history.replaceState`, `URLSearchParams`
    // — everything the URL parsers in `state.ts` reach for. Lighter and
    // faster than jsdom for our needs (no Pixi/Canvas in tests).
    environment: "happy-dom",
    include: ["src/**/*.test.ts"],
    // Don't try to ship the whole web app's worker / wasm graph through
    // Vite's optimisation — just transpile what the tests touch.
    server: {
      deps: {
        // Force these to be resolved by Vite (rather than node) so the
        // JSON import in `shortIds.ts` resolves the same way it does in
        // the browser bundle.
        inline: [/\.json$/],
      },
    },
  },
});
