# Plugin API v2 design (proposal)

Status: **proposal; the bounded decoration cache and its validation (`crates/extensions/src/decorations.rs`) exist, nothing else is wired yet.** Goal: a plugin author can build what an Equicord
plugin does, as a native Wasm plugin written from scratch (Rust, C, C++ or anything that
targets Wasm). Equicord plugins are not loaded or ported. Low RAM is the priority: the
client should stay under 500 MB, with 600 MB acceptable and 700 MB+ a failure.

## Why v1 is too limited

v1 plugins only answer a user action or app event with one JSON result. They cannot:

- put anything into the UI (badges, buttons, menu entries, indicators, settings pages);
- change how a message is shown, or what is sent;
- run on a timer or in the background;
- fetch data from the network;
- act without a per-effect Apply click;
- hold more than a single 1 MiB string, or more than 8 installed plugins (now 64).

## Constraints that stay

- No Wasm call from UI rendering or audio callbacks. The UI only reads host-side caches.
- Every cache, queue and payload is bounded by bytes and item count.
- Plugins never see tokens, files or arbitrary network. No telemetry.
- Each call still gets a fresh, fuel- and memory-limited Wasm instance, freed afterwards.
  This is the main reason plugins cost almost no resident RAM.

## Implemented so far

- `decorations` capability. A `message_event` handler returns `decoration` (SDK:
  `DecorationOutput`); the host validates it, stores it in the bounded cache and the timeline
  paints badges/prefix/suffix beside the author name in the normal (non-compact) layout.
- `presentation` (blur/collapse/hide) replaces the body with a click-to-show placeholder; the
  row-height cache includes that state so the virtualized timeline stays correct.
- Decorations are dropped when their plugin is disabled and on every account change.
- Not yet: compact layout, user/server decorations, a real visual blur, and a plugin
  invocation for messages that were already loaded before the plugin was enabled.

## New plugin surfaces

| Surface | What the plugin returns | How it stays cheap |
| --- | --- | --- |
| `decorate_message` | Per-message badges, prefix/suffix text, text replacement, blur/hide/collapse flag, extra native elements under the message | Host calls it off-thread in batches when messages load; results cached per message ID (LRU, byte-capped); UI only reads the cache |
| `decorate_user` | Badges/icons/colour next to a name in chat, member list, profile, voice list | Same cache, keyed by user ID |
| `decorate_server` | Dot/badge/tooltip on server and channel list entries | Same cache, keyed by guild/channel ID |
| `before_send` | Replacement text, attachments to drop, silent flag, or cancel | Runs once when the user presses Send; Send is the consent |
| `command` | Draft text, a message to send, or a panel | Declared name/arguments; runs when the user invokes it |
| `contribute_ui` | Static manifest entries: chat-bar button, message-popover button, context-menu item, toolbar item, settings page | No Wasm until clicked; zero per-frame cost |
| `schedule` | Runs at a declared interval (minimum enforced by host, e.g. 15 s) | Host timer, one call at a time, stopped when the account logs out |
| `on_event` | Gateway-derived events: typing, voice join/leave, presence, relationship changes, message edit/delete | Already exists as `app_event`; extended with the new event kinds |

## New capabilities

- `auto_apply:<effect>`: user grants specific low-risk effects (for example `mark_read`,
  `set_reaction`, `copy_text`, `notice`) to run without a click, per plugin, revocable. The
  default stays "needs Apply". Never available for deletion, moderation, role/server changes,
  account edits or sending messages on its own. Rate-limited, and each auto-applied effect is
  shown in a bounded in-app activity list.
- `network:<host>`: **pull only.** Host-performed HTTPS `GET` to hosts named in the manifest
  and granted by the user. No request body, cookies, credentials or custom auth headers; the
  host never adds Discord data to a request. Response capped (for example 256 KiB), timeout,
  per-plugin request budget. Wasm receives bytes, never a socket. Because a URL itself can
  carry data, the consent screen lists the hosts and says plugin-chosen URL text goes to them.
  There is no POST/PUT. Serein itself sends no analytics or telemetry; data collection stays zero.
- `storage_v2`: key-value store with per-key and total quotas (replaces the single string).
- `decorate_*` grants are separate for message, user and server data so consent stays specific.

## Memory budget

| Item | Cap (proposal) |
| --- | --- |
| Decoration cache | 4 MiB total across plugins, evicted LRU |
| Per-plugin storage | 1 MiB (as today) |
| Wasm instance | Existing per-call limit, released after the call |
| Plugins installed | 64 per scope (already changed) |
| Plugins active | No fixed count; total decoration work is budgeted per frame batch |

Authors carry part of the cost: a plugin that is slow or returns large decorations is
dropped by the fuel and byte limits, and the host shows which plugin caused it.

## What an Equicord-style plugin maps to

| Equicord examples | v2 surface |
| --- | --- |
| PlatformIndicators, ShowBadgesInChat, GlobalBadges, FriendshipRanks, ShowConnections | `decorate_user` (+ `network` for badge databases) |
| MessageLatency, MessageFetchTimer, ClearURLs, SendTimestamps, BlurNSFW, ShowHiddenThings, ImageFilename | `decorate_message` |
| SilentTyping, SilentMessageToggle, SplitLargeMessages, TextReplace, SedEnhanced, MoreCommands, TypingTweaks | `before_send`, `command`, `contribute_ui` |
| ChannelTabs, CollapsibleUI, BetterFolders, CommandPalette, EquicordToolbox, Quoter | `contribute_ui` + panels (some need new native view support) |
| CallTimer, VoiceChatUtilities, FollowVoiceUser, VolumeBooster, WhosWatching, MemberCount | `decorate_*` + `on_event` + existing voice effects |
| ScheduledMessages, RelationshipNotifier, ClientSideBlock, MessageLogger(Enhanced) | `schedule`, `on_event`, `auto_apply`, storage |
| Timezones, UserPFP, Translate, BigFileUpload, Quoter, YoutubeDescription | `network` + decorate/command |
| CustomRPC, RPCEditor, music/game presence plugins | existing `rich_presence` (+ `network` for now-playing APIs) |
| ClientTheme, FontLoader, ThemeAttributes, themes | existing themes and `appearance` |

## What will not be supported

These depend on patching Discord's web client, defeating protections, or acting beyond what
this client may do, so no equivalent will be offered:

- Experiments, ConsoleShortcuts, WebpackTarball, ReactErrorDecoder, internal-module patching.
- FakeNitro/stream-quality bypasses, SpotifyCrack, PlatformSpoofer, Quest auto-completers,
  NoTrack-style toggles for services Serein never contacts.
- Showing channels, data or permissions the server did not send to the account.
- Reading credentials or running arbitrary code outside the Wasm sandbox.

## Phases

1. `contribute_ui` (buttons, menu items, settings pages) and raising limits. Safest, no
   rendering changes.
2. `decorate_message` / `decorate_user` / `decorate_server` with the byte-capped cache.
3. `before_send` and `command`.
4. `schedule`, `auto_apply`, `storage_v2`.
5. `network` (needs a threat-model update first).

Each phase ships with a SDK version bump, updated docs and synthetic tests (bounds,
malformed output, consent checks). Existing v1 plugins keep working; the `api_version`
field selects behaviour.

## Decisions (owner)

- Exclusions above accepted; FakeNitro and Quest auto-completers are not wanted.
- Host-performed `network` is accepted as pull-only; sending data out is not.
- `auto_apply` exists in the limited form above (maintainer's call).
- The goal is an SDK others can build almost anything with, not bundled plugins.
