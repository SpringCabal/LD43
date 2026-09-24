# To the Last Drop — Core-WASM guest modules

Three guest crates compile to `wasm32-unknown-unknown` and load through
`LuaRules/wasm/manifest.txt` and `LuaUI/wasm/manifest.txt`:

| Crate | Environment | Installed artifact |
| --- | --- | --- |
| `rules-synced` | rules-synced | `LuaRules/wasm/ttld-rules.wasm` |
| `rules-unsynced` | rules-unsynced | `LuaRules/wasm/ttld-rules-unsynced.wasm` |
| `ui` | ui | `LuaUI/wasm/ttld-ui.wasm` |

- `rules-synced`: the game rules as spring-addons gadgets (`src/gadgets/`),
  and the unit scripts as a Rust Core-CUS module (`src/unit_scripts/`).
- `rules-unsynced`: draws units at the scale their def's `hscale`/`vscale`
  custom params ask for, like the Lua build's `unit_enlarger.lua`.
- `ui`: the HUD (RmlUi), camera, input, lighting, fire and minimap widgets.

Lua remains only for game definitions and assets; the original Lua game is the
separate `ToTheLastDrop.sdd` checkout. `GameData/weapondefs_post.lua` copies
each weapon's `rgbcolor` and `size` into the custom params `projectile_rgb` and
`projectile_size`, because the weapon def info has no visual size or colour,
and the projectile lights need them.

Run `just copy` here to format, lint, build, and install the release modules.
The game loads the installed artifacts, not the files under `target/`.

From the SpringCabal workspace root:

```sh
just verify ttld headless     # gameplay assertions
just verify ttld visual       # screenshots against goldens, boss size, hover, selection
just verify ttld lua-visual   # the Lua build's boss size and hover, for reference
just verify ttld pose-parity  # Lua/Rust spell piece transforms
./run-wasm.sh --game ttld
```

Verification builds its own feature-enabled modules into its cache and never
touches the installed ones.
