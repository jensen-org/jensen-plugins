# Building Jensen plugins

Jensen is extensible. A plugin can add commands, panels, whole views, and themes, and it can read the
project's code graph, query the knowledge index, inspect git history, and read or write files, all
without ever touching your machine directly.

That last part is the whole design. Jensen runs as a **kernel**: it is the only actor that ever
touches the filesystem, the network, git, or the OS. Your plugin is an unprivileged sandbox with zero
ambient authority. It cannot open a file, a socket, or a process, cannot reach the app's UI, and
cannot see or talk to another plugin. The only way it causes any effect is to ask the kernel to
perform an action on its behalf, a brokered "syscall" that the kernel checks against the permissions
you declared and the user consented to, meters against a resource budget, performs itself, and can
deny or revoke at any time. Deny by default. If the kernel does not implement a syscall, that power
does not exist for any plugin.

This guide takes you from an empty folder to a plugin published in this registry.

## Table of contents

- [The security model](#the-security-model)
- [Anatomy of a plugin](#anatomy-of-a-plugin)
- [The manifest](#the-manifest)
- [Getting started](#getting-started)
- [Writing the logic](#writing-the-logic)
- [Writing the UI](#writing-the-ui)
- [Examples](#examples)
- [Resource limits](#resource-limits)
- [Publishing](#publishing)
- [Security checklist](#security-checklist)

## The security model

Two sandboxes, one broker:

- **Logic** runs as WebAssembly in a memory-isolated sandbox with no filesystem and no network of its
  own. It reaches the outside world only through a single host call, `jensen_syscall`.
- **UI** runs in a separate null-origin sandboxed iframe. It has no network (`connect-src 'none'`),
  no access to the app, and no way to call the kernel. It talks only to its own logic half, by posting
  messages that the host relays.
- **Every effect** flows through the kernel: `charge the resource budget → check the capability →
  perform the action → return a result`. A denied call returns an error, it never crashes the app, and
  the user is shown which capability was blocked so they can grant or keep it blocked.

Because everything a plugin can touch is declared up front in `manifest.json`, Jensen renders your
commands, panels, and themes **without running any of your code**. Your WASM executes only when a
command or panel is actually used, and when the plugin is disabled every registration is reclaimed.

## Anatomy of a plugin

A plugin is a directory with up to a few parts:

```
my-plugin/
  manifest.json        required: identity, contributions, permissions
  plugin.wasm          the compiled logic (omit for a theme-only plugin)
  ui/                  the iframe UI (optional)
    index.html
    main.js
  theme.json           a theme's tokens (only for a theme contribution)
```

- `manifest.json` is the contract. It is the same file Jensen validates, the page shows, and the
  registry linter checks, so they can never disagree.
- `plugin.wasm` is your logic compiled to `wasm32-unknown-unknown`.
- `ui/` is a plain web page. It renders inside the sandboxed iframe.
- Any theme tokens file is delivered to your plugin directory and read by name.

The published example plugins in this repo are the shortest way in: `plugins/hello` (command + panel +
UI), `plugins/impact` (reads the code graph), `plugins/plan-lint` (a markdown renderer, no UI), and
`plugins/theme-light` (a theme with no code at all).

## The manifest

```json
{
  "id": "dev.you.myplugin",
  "name": "My Plugin",
  "version": "0.1.0",
  "minAppVersion": "0.0.0",
  "description": "One line describing what it does.",
  "author": "Your Name",

  "entry": { "wasm": "plugin.wasm", "ui": "ui/index.html" },

  "contributes": {
    "commands": [{ "id": "my.run", "title": "Run My Thing", "category": "My Plugin" }],
    "panels":   [{ "id": "my.panel", "label": "My Panel", "whenView": "code" }],
    "views":    [{ "id": "my.view", "label": "My View", "icon": "sparkles" }],
    "themes":   [{ "id": "my.theme", "name": "My Theme", "dark": true, "tokensFile": "theme.json" }]
  },

  "permissions": {
    "graph": false,
    "knowledge": false,
    "git": false,
    "fs": [],
    "network": []
  },

  "activationEvents": ["onCommand:my.run", "onPanel:my.panel"]
}
```

Field reference:

| Field | Meaning |
| --- | --- |
| `id` | Reverse-DNS, lowercase letters, digits, `.` and `-` only. Must be globally unique. |
| `version` / `minAppVersion` | Semver. The plugin is rejected if `minAppVersion` is newer than the running app. |
| `entry.wasm` | Path to the compiled logic, relative to the plugin dir. Omit for a theme-only plugin. |
| `entry.ui` | Path to the iframe entry. Omit if the plugin has no UI. Presence of this field is what grants the `ui.post` syscall. |
| `contributes.commands` | `{ id, title, category?, icon? }`. Appear in the command palette and native menu. |
| `contributes.panels` | `{ id, label, whenView?, icon? }`. Dock into the right panel; `whenView` limits them to one view (for example `"code"`). |
| `contributes.views` | `{ id, label, icon }`. Top-level views. |
| `contributes.themes` | `{ id, name, dark, tokensFile }`. `tokensFile` is a JSON token map in the plugin dir. |
| `permissions.graph` / `knowledge` / `git` | Booleans. Gate the matching syscalls. |
| `permissions.fs` | Array of **relative** subpaths (for example `["docs/"]`). Each confines file access to `<project root>/<subpath>`. `..`, absolute paths, and backslashes are rejected. |
| `permissions.network` | Array of **exact** hosts (for example `["api.example.com"]`). No wildcards, no ports needed. |
| `activationEvents` | When to load the WASM: `onStartup`, `onCommand:<id>`, `onPanel:<id>`, `onView:<id>`. Every event must reference a contribution you declared. A theme-only plugin has none. |

The `permissions` you write are only what you *request*. The kernel enforces the **granted** set, which
is your request intersected with what the user approved on the consent screen. A user can install your
plugin and still withhold, say, network, and the affected syscalls return a permission error.

## Getting started

Prerequisites: a Rust toolchain and the WASM target.

```bash
rustup target add wasm32-unknown-unknown
```

Copy `plugins/hello/` as your starting point. Its `Cargo.toml` is the minimum:

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[dependencies]
extism-pdk = "1"
# The PDK ships in the Jensen app repo. Depend on it by git for a standalone plugin repo:
jensen-plugin = { git = "https://github.com/jensen-org/jensen" }
serde_json = "1"

[lib]
crate-type = ["cdylib"]
```

Build it and sideload it for local testing (sideloading does **not** require a checksum, only registry
installs do):

```bash
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/my_plugin.wasm plugin.wasm

# Jensen discovers plugins under its state dir. JENSEN_STATE_DIR overrides it;
# otherwise it is ~/.local/state/jensen on Linux and the platform equivalent elsewhere.
mkdir -p "$JENSEN_STATE_DIR/plugins/dev.you.myplugin"
cp -r manifest.json plugin.wasm ui "$JENSEN_STATE_DIR/plugins/dev.you.myplugin/"
```

Restart Jensen, open the Plugins view, enable the plugin, and approve its permission scorecard. Your
command shows up in the palette, your panel in the right panel, your theme in the theme list. Disable
it and all of that disappears in one tick.

## Writing the logic

Your `lib.rs` exports functions with `#[plugin_fn]`. Two matter:

- `activate()` runs once when the plugin loads. Do setup here.
- `handle_command(input: String)` runs every time one of your commands or UI actions fires. The input
  is JSON; return JSON.

Reach the kernel through the `jensen-plugin` PDK. Every helper is a brokered syscall and returns an
`Err` if the capability is not granted:

```rust
use jensen_plugin::{graph, knowledge, git, fs, net, ui, log};

graph::impact("src/auth.rs")?;              // needs permissions.graph
graph::dependencies("src/auth.rs", true)?;  // incoming = true
graph::query("...")?;
knowledge::search("how does login work")?;  // needs permissions.knowledge
git::history(Some("src/auth.rs"), 20)?;     // needs permissions.git
fs::read("docs/spec.md")?;                  // needs a matching permissions.fs scope
fs::write("docs/out.md", contents)?;
fs::data_read("cache.json")?;               // your private data dir, always allowed
fs::data_write("cache.json", contents)?;
net::fetch("api.example.com", "/v1/status")?; // needs the host in permissions.network
ui::post("topic", serde_json::json!({ ... }));  // push an event to your iframe
log::info("hello");
```

For anything not covered by a helper (for example `knowledge.ingest`, `git.semanticDiff`,
`graph.explainService`), call the raw broker: `jensen_plugin::syscall("git.semanticDiff", params)`.

## Writing the UI

The UI is an ordinary web page that runs in a null-origin iframe. It cannot fetch, cannot reach the
app, and cannot call the kernel. Its only channel is a typed message bridge to its own logic half.

Use the `@jensen/plugin-ui` client:

```js
import { PluginUI } from "@jensen/plugin-ui";

const jensen = new PluginUI();

document.getElementById("run").addEventListener("click", async () => {
  const result = await jensen.invokeCommand("my.run", { some: "args" });
  document.getElementById("out").textContent = JSON.stringify(result);
});

// Events your logic pushes with ui::post arrive here.
jensen.on("progress", (payload) => {
  document.getElementById("out").textContent = JSON.stringify(payload);
});
```

`invokeCommand(command, args)` is relayed to your WASM `handle_command`. Whatever it returns resolves
the promise. `on(topic, handler)` receives everything your logic sends with `ui::post`. If you would
rather not add a dependency, the raw protocol is three message shapes posted to `window.parent`:
`{ kind: "req", id, method: "invokeCommand", params }` out, and `{ kind: "res", id, ok, result }` /
`{ kind: "evt", topic, payload }` in. See `plugins/hello/ui/` for the dependency-free version.

## Examples

### 1. A command with no UI and no permissions

The smallest useful plugin.

```json
{
  "id": "dev.you.greet",
  "name": "Greet", "version": "0.1.0", "minAppVersion": "0.0.0",
  "description": "Says hello.", "author": "You",
  "entry": { "wasm": "plugin.wasm" },
  "contributes": { "commands": [{ "id": "greet.say", "title": "Say Hello" }] },
  "permissions": {},
  "activationEvents": ["onCommand:greet.say"]
}
```

```rust
use extism_pdk::*;
use serde_json::{json, Value};

#[plugin_fn]
pub fn handle_command(input: String) -> FnResult<String> {
    let request: Value = serde_json::from_str(&input).unwrap_or(Value::Null);
    let command = request.get("command").and_then(Value::as_str).unwrap_or("");
    let result = match command {
        "greet.say" => json!({ "message": "Hello from the sandbox!" }),
        other => json!({ "error": format!("unknown command '{other}'") }),
    };
    Ok(result.to_string())
}
```

### 2. A panel with a UI round-trip

Add `entry.ui` and a panel, and push an event back to the iframe. This is `plugins/hello`.

### 3. Reading the code graph

Request `graph` and answer a question about impact. This is `plugins/impact`.

```json
{
  "permissions": { "graph": true },
  "contributes": { "commands": [{ "id": "impact.check", "title": "Impact of File" }] },
  "activationEvents": ["onCommand:impact.check"]
}
```

```rust
#[plugin_fn]
pub fn handle_command(input: String) -> FnResult<String> {
    let request: serde_json::Value = serde_json::from_str(&input).unwrap_or_default();
    let target = request["args"]["path"].as_str().unwrap_or("");
    let impact = jensen_plugin::graph::impact(target)
        .unwrap_or_else(|e| serde_json::json!({ "error": e.to_string() }));
    Ok(impact.to_string())
}
```

### 4. Calling an allowed network host

Declare the exact host. The kernel performs the HTTPS request; your plugin never holds a socket, and
redirects to any other host are refused.

```json
{
  "permissions": { "network": ["api.example.com"] },
  "contributes": { "commands": [{ "id": "status.fetch", "title": "Fetch Status" }] },
  "activationEvents": ["onCommand:status.fetch"]
}
```

### 5. A theme, with no code at all

A theme-only plugin needs no WASM and no activation events. Ship a token map and declare it. This is
`plugins/theme-light`.

```json
{
  "id": "dev.you.midnight",
  "name": "Midnight", "version": "0.1.0", "minAppVersion": "0.0.0",
  "description": "A dark theme.", "author": "You",
  "entry": {},
  "contributes": {
    "themes": [{ "id": "midnight", "name": "Midnight", "dark": true, "tokensFile": "theme.json" }]
  },
  "permissions": {},
  "activationEvents": []
}
```

`theme.json` is a map of the app's CSS design tokens to your values; override every token you want
changed (a partial map leaves the base theme showing through). See `plugins/theme-light/theme.json`
for the full set.

## Resource limits

Every plugin runs under a resource governor. The defaults per plugin are:

| Limit | Default | Meaning |
| --- | --- | --- |
| CPU timeout | 5 s | A single syscall or command that runs longer is aborted. |
| Memory | 256 pages (16 MiB) | The WASM instance cannot grow past this. |
| Syscall rate | 50 / s | A token bucket. Bursts beyond it are rejected, not queued forever. |
| Concurrency | 4 | At most this many syscalls in flight at once. |

If a plugin storms the rate limit, loops forever, or blows the memory cap, the governor stops it and
the user sees a toast explaining why. The user can also force-stop any plugin. Write your logic to do
bounded work per command and to handle a denied or rate-limited syscall gracefully.

## Publishing

### How installation works

Users install from this registry: a public `index.json` that lists, per plugin,
`{ id, name, author, description, category, version, min_app_version, repo, sha256 }`. The registry
locates a plugin (`repo` + `version`) and pins it (`sha256` is the checksum of the release
`manifest.json`); it carries no capabilities and no code. Each plugin's `repo` hosts GitHub releases.
To install, Jensen builds the asset URLs as
`https://github.com/<repo>/releases/download/<version>/<asset>`, downloads the release `manifest.json`
and checks it against `sha256`, validates it, confirms its id matches the registry entry, downloads
each artifact the manifest's `dist` block names (`plugin.wasm`, `ui.zip`, and any theme tokens) and
checks each against the hash the now-trusted manifest pins, and stages the plugin disabled until the
user approves its permissions. There are no keys to manage: because the manifest pins each artifact's
checksum, the one entry hash vouches for the whole download, and what a plugin may actually do is
enforced by the sandbox and the consent screen, not the registry.

### Assemble a release

`tools/release-plugin.sh` does the whole thing: builds the WASM (when the manifest declares one), zips
`ui/`, copies each theme tokens file, computes the artifact sha256s, writes a release `manifest.json`
with a `dist` block, and prints the manifest's own sha256.

```bash
tools/release-plugin.sh my-plugin/
# writes my-plugin/release/{manifest.json, plugin.wasm?, ui.zip?, theme.json?}
# prints the manifest sha256 to put in the registry entry
```

The release `manifest.json` is your manifest plus a `dist` block, for example:

```json
"dist": {
  "wasm": { "asset": "plugin.wasm", "sha256": "..." },
  "ui":   { "asset": "ui.zip",     "sha256": "..." },
  "assets": [{ "asset": "theme.json", "sha256": "..." }]
}
```

Because the manifest pins every artifact's sha256, its own checksum vouches for the whole download.
Create a GitHub release whose tag equals the manifest `version`, and upload the files from `release/`
as assets.

### Submit to the registry

Open a pull request adding one entry to the `plugins` array in `index.json`:

```json
{
  "id": "dev.you.myplugin",
  "name": "My Plugin",
  "author": "You",
  "description": "One line describing what it does.",
  "category": "lint",
  "version": "0.1.0",
  "min_app_version": "0.0.0",
  "repo": "you/my-plugin",
  "sha256": "the manifest sha256 release-plugin.sh printed"
}
```

`version` must equal the release tag, `id` must match your release manifest, and `sha256` is the value
`release-plugin.sh` printed. Validate the change against `schema.json`, and run the same validator
Jensen uses, `pluginhost-lint` (from the app repo):

```bash
cargo run -q -p pluginhost --bin pluginhost-lint -- my-plugin/manifest.json
```

Once merged, your plugin appears in every user's Plugins view. See `CONTRIBUTING.md` for the checklist.

## Security checklist

- Request the **least** you need. Every capability you declare is shown to the user and raises your
  risk score. A plugin that asks for nothing installs with no friction.
- Scope `fs` to the narrowest subpath that works, never the whole project.
- List **exact** network hosts. There are no wildcards, and the kernel will not follow a redirect to
  any host you did not list.
- Keep per-command work bounded so you stay inside the CPU and rate budgets.
- Never assume a syscall succeeds. A user can withhold any capability, and your plugin should degrade,
  not crash.
