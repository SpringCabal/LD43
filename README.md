# ToTheLastDrop WASM

The Rust/WASM port of the game.

Use the [SpringCabal development repository](https://github.com/gajop/springcabal-dev) for engine setup, launching, builds and verification.

## Scoped SDK port

Engine calls use `spring-sdk` through explicit Spring references. Gadgets own
unit scripts alongside game state; queued work receives a fresh reference.
Unit-script tasks keep their unit context across waits and acquire Spring only
for synchronous engine operations. The CUS timer stays after all queued frame
commands, including their continuations. Synced random draw order is preserved.

The development repository supplies the headless gameplay, visual,
Lua/Rust pose comparison and controls suites. The gameplay suite applies real
engine damage to a house: 400 damage must not ignite it; 401 damage must.
The native unit script registers `WorldHitByWeapon`, the callback used by engine
damage delivery. Its handler leaves the damage unchanged and sets `on_fire`
above the same threshold as Lua's `HitByWeapon` handler.

World health bars use the original Lua renderer alongside the Wasm HUD and
input. Both editions share its shaders, visibility rules, bar states and
defaults. Gameplay and unit scripts remain in Wasm.

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
