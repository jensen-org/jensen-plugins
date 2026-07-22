# Contributing a plugin

This registry lists plugins Jensen can install. To add yours, publish a release in your own repo and
open a pull request adding one entry here. Read `AUTHORING.md` for the full guide; this is the
checklist.

## Checklist

1. **Build the release.** From your plugin's source directory:

   ```bash
   tools/release-plugin.sh my-plugin/
   ```

   It builds the wasm (when your manifest declares one), zips `ui/`, copies any theme tokens file,
   writes `my-plugin/release/manifest.json` with a `dist` block pinning every artifact's sha256, and
   prints the manifest's own sha256, the value for your registry entry.

2. **Validate the manifest** with the same linter Jensen uses (from the Jensen app repo):

   ```bash
   cargo run -q -p pluginhost --bin pluginhost-lint -- my-plugin/manifest.json
   ```

3. **Cut a GitHub release** in your plugin's repo whose tag equals the manifest `version`, uploading
   the files from `release/`: `manifest.json`, plus `plugin.wasm`, `ui.zip`, and any theme tokens when
   your plugin has them.

4. **Add one entry** to the `plugins` array in `index.json` (see `README.md` for the shape). `id` must
   match your release manifest, `version` must equal the release tag, `repo` is the GitHub `owner/name`
   hosting the release, and `sha256` is the value `release-plugin.sh` printed.

5. **Validate `index.json`** against `schema.json`, then open the pull request.

## First-party plugins

The plugins under `plugins/` are maintained here. Each is a normal plugin source directory; assemble a
release the same way with `tools/release-plugin.sh plugins/<name>`.
