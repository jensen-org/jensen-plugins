# Jensen plugin registry

The catalog of plugins Jensen can install from its **Plugins** page. Jensen fetches `index.json` from
this repository's raw URL, the same way it fetches the language-server registry, so the catalog lives
here and never ships inside the app.

Hosting is a public GitHub repository served over `raw.githubusercontent.com`. There is no server and
no cost.

## Layout

```
index.json        the catalog Jensen fetches
schema.json       the JSON Schema for index.json
AUTHORING.md      the full plugin authoring guide (TypeScript)
CONTRIBUTING.md   how to publish a plugin here
plugins/          source for the first-party example plugins (Rust/WASM advanced path)
  hello/          command + panel + UI starter
  impact/         reads the code graph (a graph tool)
  plan-lint/      a markdown fenced-block renderer, no UI
  theme-light/    a light theme, no code at all
```

Plugins are written in **TypeScript** and published with `jensen publish`, which generates the manifest
and assembles the release for you (see `AUTHORING.md`). The `plugins/` examples are the advanced
Rust-to-WebAssembly path, still supported and published with the same one command.

## How the app reads this

Jensen fetches:

```
https://raw.githubusercontent.com/jensen-org/jensen-registry/develop/index.json
```

If the file is missing or unreachable, the Plugins page shows a calm empty state, never an error. The
catalog only lists what is available; installed plugins live on the user's machine. A user can turn the
public registry off, or point Jensen at a private registry URL, from Settings → Plugins.

## What an entry is

The registry tells Jensen where to find a plugin (`repo` + `version`) and pins it with the checksum of
its release manifest (`sha256`). It carries no capabilities and no code. Integrity is verified at
install: Jensen downloads the release `manifest.json`, checks it against `sha256`, validates it, and
unpacks the plugin (its `main.js`, or a WASM module and UI for the advanced path), staged disabled
until the user approves its permissions. What the plugin may actually do is enforced by the sandbox and
the consent screen, not by the registry.

## Publishing a plugin

See `CONTRIBUTING.md` for the checklist and `AUTHORING.md` for the full guide. In short: run
`jensen publish` in your built plugin directory (it generates `manifest.json`, assembles a `release/`
folder, and prints the entry), cut a GitHub release in your own repo whose tag equals the `version`
uploading every file in `release/`, then open a pull request adding one entry to the `plugins` array in
`index.json`.

An entry:

```json
{
  "id": "acme.markdown-lint",
  "name": "Markdown Lint",
  "author": "Acme",
  "description": "Flags style issues in Markdown as you type.",
  "category": "lint",
  "version": "1.2.0",
  "min_app_version": "0.1.0",
  "repo": "acme/md-lint",
  "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
}
```

- `id` is reverse-dns `publisher.name`, lowercase, stable across versions. It is the install directory
  name and must match the id in the release manifest.
- `repo` is the GitHub `owner/name` that hosts the release. Jensen builds the asset URLs as
  `https://github.com/<repo>/releases/download/<version>/<asset>`.
- `version` must equal the release tag.
- `sha256` is the checksum `jensen publish` printed for `manifest.json`.

`schema.json` is the JSON Schema for `index.json`; validate your change against it before opening the
PR. `jensen publish` already validates the manifest it generates.
