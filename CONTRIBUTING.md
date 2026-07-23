# Contributing a plugin

This registry lists plugins Jensen can install. To add yours, publish a release in your own repo and
open a pull request adding one entry here. Read `AUTHORING.md` for the full guide; this is the
checklist.

## Checklist

1. **Build and publish.** From your built plugin directory (the one with `main.js`):

   ```bash
   npm run build
   jensen publish
   ```

   `jensen publish` reads `package.json`, generates and validates `manifest.json`, computes its
   sha256, and prints the registry entry to add here. (Or use **Plugins → Publish** in the app.)

2. **Cut a GitHub release** in your plugin's repo whose tag equals the `version`, uploading
   `manifest.json` and `main.js` (plus a zipped `ui/` if you have one).

3. **Add one entry** to the `plugins` array in `index.json` (see `README.md` for the shape). `id` must
   match your manifest, `version` must equal the release tag, `repo` is the GitHub `owner/name` hosting
   the release, and `sha256` is the value `jensen publish` printed.

4. **Validate `index.json`** against `schema.json`, then open the pull request.

## First-party plugins

The plugins under `plugins/` are the Rust-to-WebAssembly examples, maintained here. Assemble a release
for one with `tools/release-plugin.sh plugins/<name>`.
