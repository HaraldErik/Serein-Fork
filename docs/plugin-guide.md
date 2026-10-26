# Writing Serein plugins

Plugins are small WebAssembly (Wasm) modules that Serein runs in a sandbox. You can write them
in Rust (best supported, has an SDK), or C/C++ or any language that compiles to Wasm and speaks
the same buffer/JSON contract. Plugins are **not** Vencord/Equicord plugins and do not patch
Discord's web client; Serein is a native client, so plugins use a documented host API instead.

This guide covers the model, a first plugin, the manifest, permissions, limits, and the
decorations API. Deep references: [SDK overview](extension-sdk-overview.md),
[data reference](extension-sdk-reference.md), [actions reference](extension-sdk-actions.md),
[all capabilities](extensions.md#capability-reference), and the
[v2 roadmap](plugin-api-v2.md).

## How a plugin runs

1. **Declare.** A manifest names the plugin, its actions and the permissions (capabilities) it needs.
2. **Grant.** When you enable the plugin, Serein shows each permission. Nothing is granted silently.
3. **Invoke.** A click, a message event, or an app event starts a **fresh** Wasm instance. Serein
   passes JSON describing the action and the data you were granted.
4. **Return.** Your handler returns one JSON result: a panel to show, text to put in the draft, a
   decoration for a message, or a proposed action.
5. **Discard.** The instance is freed. Only the one `storage` string you save persists.

Why this is cheap on RAM: no plugin process stays alive, nothing runs while the UI draws, and every
call has hard limits.

| Limit | Value |
| --- | --- |
| Memory per call | 16 MiB |
| CPU budget per call | 10,000,000 Wasm fuel units |
| Input and output JSON | 256 KiB each |
| Wasm module | 4 MiB; whole package 16 MiB |
| Saved storage | one UTF-8 string, 1 MiB |
| Installed plugins | 64 per scope |
| Decoration cache | 2,048 entries / 4 MiB, oldest evicted first |

A plugin cannot read your token, open files, use the network or call Discord directly. Actions that
change something (send, edit, delete, react, join a call…) are **proposals** that wait for your
click on **Apply**.

## Your first plugin (Rust)

Requirements: the Rust toolchain and the Wasm target.

```sh
rustup target add wasm32-unknown-unknown
```

Work inside `examples/extensions/` so the SDK path and workspace already exist, or copy an example
such as [`message-counter`](../examples/extensions/message-counter) or
[`link-badges`](../examples/extensions/link-badges).

`Cargo.toml`

```toml
[package]
name = "hello"
version = "1.0.0"
edition = "2024"
license = "MIT OR Apache-2.0"

[lib]
crate-type = ["cdylib"]

[dependencies]
serein-extension-sdk = { path = "../../extensions/sdk" }

# Keeps the module small; the examples use the same settings.
[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

`manifest.json`

```json
{
  "api_version": 1,
  "id": "hello",
  "name": "Hello",
  "version": "1.0.0",
  "author": "You",
  "license": "MIT OR Apache-2.0",
  "source": "https://github.com/you/hello",
  "kind": "plugin",
  "capabilities": ["app_context"],
  "actions": [
    {"id": "show", "label": "Show current channel", "surface": "panel"}
  ]
}
```

`src/lib.rs`

```rust
use serein_extension_sdk::{AppInvocation, AppOutput, Element, Output};

fn handle(input: AppInvocation) -> AppOutput {
    let channel = input
        .app
        .and_then(|app| app.context)
        .and_then(|context| context.channel)
        .map_or("no channel selected".to_string(), |channel| channel.name);
    AppOutput {
        output: Output {
            panel: vec![Element::Text { text: format!("Current channel: {channel}") }],
            ..Default::default()
        },
        ..Default::default()
    }
}
serein_extension_sdk::export!(handle);
```

Build, package and try it offline:

```sh
cargo build --release --target wasm32-unknown-unknown
python extensions/pack.py hello/manifest.json target/wasm32-unknown-unknown/release/hello.wasm hello.serein-extension
cargo run --locked --features demo -- --demo
```

In **Settings → Extensions → Import**, pick `hello.serein-extension`, review the permission and
enable it, then open the plugin card. Imported packages are marked unreviewed. `AppInvocation` and `AppOutput` carry the app data and
host proposals; the plain `Invocation`/`Output` pair is enough for plugins that only use panels,
storage or the draft. [`app-toolbox`](../examples/extensions/app-toolbox/src/lib.rs) is a larger,
tested example, and the [data reference](extension-sdk-reference.md#app-data) lists every field.

### Test without Serein

Handlers are plain functions. Call them in unit tests, or push JSON through the same path as the
Wasm export:

```rust
let bytes = serein_extension_sdk::dispatch(br#"{"action":"show"}"#, handle)?;
```

The repository check also runs real packages in the host sandbox:

```sh
cargo run --locked --release -p extensions --example sdk_check -- \
  examples/extensions/target/wasm32-unknown-unknown/release \
  extensions/target/wasm32-unknown-unknown/release
```

## Actions and surfaces

Each action has an `id`, a `label` and a `surface` that decides when it runs:

| Surface | Runs when | Typical result |
| --- | --- | --- |
| `panel` | You open the plugin and press a button | `panel` elements |
| `message` | You pick it from a message's menu | panel or proposal about that message |
| `composer` | You pick it from the composer **Tools** menu | `replacement` draft text |
| `message_event` | A message is created, edited or deleted | `storage`, or a **decoration** |
| `app_event` | Connection, navigation, voice, settings… change | `storage` |
| `activation` | The plugin is enabled or the account loads | presence, API proxy, retention flags |
| `tick` | Preview: timed appearance updates | `appearance` |
| `before_send` | Preview: you press **Send** with text in the draft | `replacement` text that is sent instead |
| `command` | Preview: you type `/<action id> arguments` and press Enter | `replacement` placed in the draft, and/or a panel or proposal |

Event surfaces cannot show panels or propose actions; they stay passive.

## Permissions (capabilities)

List what you need in `capabilities`; ask for the minimum. Examples: `app_context` (selected
channel), `timeline` (loaded messages), `message_events` (live message events), `storage`,
`composer`, `message_send`, `navigation`, `decorations`. The full table, with limits and
confirmation rules for all 54, is in the
[capability reference](extensions.md#capability-reference). Reading data uses what Serein already
loaded; it never fetches more on your behalf.

## Decorations: badges beside messages (preview)

With the `decorations` capability, a `message_event` handler can return a decoration for the
message that triggered it. Serein validates it, stores it in a bounded cache and paints it beside
the author name. Plugins are never called while the UI draws.

```rust
use serein_extension_sdk::{Badge, Decoration, DecorationOutput, EventInvocation, MessageEventKind};

fn handle(input: EventInvocation) -> DecorationOutput {
    let mut output = DecorationOutput::default();
    let Some(event) = input.message_event else { return output };
    if event.kind == MessageEventKind::Delete { return output }
    if event.content.unwrap_or_default().contains("https://") {
        output.decoration = Some(Decoration {
            badges: vec![Badge {
                text: "LINK".into(),
                tooltip: Some("This message contains a link".into()),
                color: Some(0x5865F2),
            }],
            ..Default::default()
        });
    }
    output
}
serein_extension_sdk::export!(handle);
```

Manifest: `"capabilities": ["message_events", "decorations"]` and an action with
`"surface": "message_event"`. The complete tested plugin is
[`examples/extensions/link-badges`](../examples/extensions/link-badges/src/lib.rs).

| Field | Limit |
| --- | --- |
| `badges` | up to 4; text up to 16 characters; tooltip up to 128 bytes; `color` as `0xRRGGBB` |
| `prefix` / `suffix` | up to 64 bytes each, no control characters |
| `presentation` | `blur`, `collapse` or `hide`: the message body is replaced by a "… by a plugin. Click to show." placeholder until you click it. All three currently look the same apart from the wording. |

Current gaps: decorations show only in the normal (non-compact) message layout; they only appear for
messages seen after the plugin is enabled; there are no user or server decorations yet.

## Send hooks: rewrite the draft when you press Send (preview)

An action with `"surface": "before_send"` (requires the `composer` capability) receives the draft in
`composer` every time you press **Send**. Return `replacement` to change what is sent, or nothing to
send the draft unchanged. Pressing Send is the confirmation, so no Apply click is needed.

```rust
use serein_extension_sdk::{Invocation, Output};

fn handle(input: Invocation) -> Output {
    let Some(draft) = input.composer else { return Output::default() };
    let fixed = draft.replace(" teh ", " the ");
    Output { replacement: (fixed != draft).then_some(fixed), ..Default::default() }
}
serein_extension_sdk::export!(handle);
```

How the host runs them:

- All enabled `before_send` actions run one after another, in ascending plugin-ID order. Each one
  sees the previous plugin's text.
- The message is sent once every plugin has finished. If any plugin fails, is disabled, takes longer
  than 15 seconds, or you edit the draft or switch conversation meanwhile, nothing is sent and your
  draft is kept.
- A `before_send` result may only contain `replacement` and `storage`; panels, effects and other
  outputs are rejected.
- Hooks run for typed drafts (with or without attachments). Message edits, stickers, GIFs and slash
  commands do not run them.

Complete tested example with a settings panel: [`examples/extensions/text-replace`](../examples/extensions/text-replace/src/lib.rs).

## Slash commands (preview)

An action with `"surface": "command"` (requires `composer`) becomes a slash command named after its
action `id` (lowercase letters, digits and `-`, at most 32 characters). Typing `/mock hello` and
pressing Enter runs the plugin with `action = "mock"` and `composer = "hello"`.

- A returned `replacement` **replaces the draft**; you review it and press Enter again to send. A
  command never sends a message by itself.
- Panels and Apply proposals work as for other foreground actions, so a command can also open a tool.
- Built-in commands (`/me`, `/shrug`, `/gif`, `/msg`, `/spoiler`, `/sticker`, `/tableflip`, `/unflip`)
  and server application commands you pick from the list take priority. If two plugins declare the
  same name, the one with the lowest plugin ID wins.
- If you edit the draft or switch conversation before the result arrives, it is discarded.
- Plugin commands are not yet listed in the slash-command popup; type the name yourself.

Example: [`examples/extensions/fun-commands`](../examples/extensions/fun-commands/src/lib.rs)
(`/mock`, `/reverse`, `/upper`).

## Publishing and sharing

- Keep source and license public. Share the `.serein-extension` file, or add it to the catalog under
  `extensions/` in a pull request (see [`extensions/README.md`](../extensions/README.md)).
- Imported packages are unreviewed; catalog entries are hash-pinned and reviewed by maintainers.
- Updates require new consent. Disabling removes the package and its saved data.

## What is planned

Decorations, send hooks and slash commands are the first pieces of the [Plugin API v2](plugin-api-v2.md). Next: UI entries (buttons, menu
items, settings pages), send hooks and commands, timers, storage with more keys, pull-only network
access to hosts you approve, and limited auto-apply for low-risk actions. Plugins that depend on
patching Discord's web client have no equivalent in a native client.
