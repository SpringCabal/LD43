# ToTheLastDrop WASM

The Rust/WASM port of the game.

Use the [SpringCabal development repository](https://github.com/gajop/springcabal-dev) for engine setup, launching, builds and verification.

## Scoped SDK port

Engine calls use `spring-sdk` through explicit Spring references. Gadgets own
unit scripts alongside game state; queued work receives a fresh reference.
Unit-script tasks keep their unit context across waits and acquire Spring only
for synchronous engine operations. The CUS timer stays after all queued frame
commands, including their continuations. Synced random draw order is preserved.

Verification in the dedicated port worktree:

- All 54 existing gameplay assertions pass on Linux and Wine.
- The live Lua/Rust animation comparison passes: 105 samples, identical transforms.
- Default and all-feature builds pass strict Clippy.
- The visual scenario captures all three screens and verifies both boss models.
  The renderer gate rejects Mesa package `.4` versus recorded `.2`. Two early
  captures also slightly exceed the existing 1% image limit in both the old code
  and the port; the late capture meets its 3% limit. Expected images and limits
  were not changed. The visual suite remains red.

Rendering rejects deferred lights only when their entire influence sphere is
outside the camera frustum. Mirrored terrain tiles likewise use bounds covering
the full tile and its frozen height range. Lighting, bloom, terrain detail and
geometry remain the same for visible content.

Release builds use whole-program LTO and one code generation unit. The
workspace profile applies to both installed modules and verification builds.

Startup and rendering parity: town creation runs at frame zero after event
subscription, with canonical villager order and matching placement draws.
Lua and Rust use the same deterministic per-house light masks. Per-unit scale
lookups are cached. The deferred lighting target uses RGB32F and the bloom
targets RGB8, matching Lua without an unused alpha channel. Set the optional
`skip_intro=1` game option to start in the gameplay view; the normal launch
profile retains the intro.
