# To the Last Drop — Core-WASM guest modules

Two guest crates, both compiled to `wasm32-unknown-unknown` and loaded through
`LuaRules/wasm/manifest.txt` and `LuaUI/wasm/manifest.txt`:

| crate          | environment    | ships as                      |
| -------------- | -------------- | ----------------------------- |
| `rules-synced` | `rules-synced` | `LuaRules/wasm/ttld-rules.wasm` |
| `ui`           | `ui`           | `LuaUI/wasm/ttld-ui.wasm`       |

The Lua that remains is deliberate: `scripts/*.lua` are the unit scripts, and
`LuaRules/Gadgets/api_unit_script_support.lua` covers the two things a guest
cannot do (cross-boundary sound, and the `GG.Attributes` entry point the unit
scripts call).

## Commands

    just fmt      # cargo fmt --all
    just clippy   # clippy over the workspace, warnings denied
    just build    # fmt + clippy, then a release build
    just copy     # build, then install both .wasm files into the game

`just copy` is the one to run after editing; the game loads the copied
artifacts, not the ones under `target/`.

## Validation

    ../../../../wasm-test/verify-ttld.sh headless   # 20 gameplay assertions
    ../../../../wasm-test/verify-ttld.sh visual     # renders and captures screenshots

Both rebuild the modules themselves, with the `headless-integration` and `e2e`
cargo features respectively.

## Running

    ../../../../run-wasm.sh --game ttld
