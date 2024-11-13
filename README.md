# Proto plugin for Poetry

Status: **Proof-of-Concept**

For evaluating whether [Proto](https://moonrepo.dev/proto)/[Moon](https://moonrepo.dev/moon) is a option for managing tools and run-times.

Requirements include being able to manage [Poetry](https://python-poetry.org/).

## Limitations

This plugin is a Proof-of-Concept only:

- Misuses the concept of Proto
  - Plugin is not isolated: requires Python to be already installed
  - Plugin is not a single binary/tool: [Poetry gets installed via pip](https://python-poetry.org/docs/#ci-recommendations)
- I'm not firm in Rust, so there may be dragons
- I no nothing about WASM, so there will be dragons

## Usage

Exemplary `.prototools`:

```toml
python = "3.12.7"
# Order matters: install Python first!
poetry = "1.8.2"

[plugins]
poetry = "github://malte-behrendt/proto-plugin-poetry"
```

## Manual test

See [WASM Plugin Documentation](https://moonrepo.dev/docs/proto/wasm-plugin#testing).

```bash
proto install
cargo wasi build
proto --log trace list-remote poetry-test
proto --log trace install poetry-test 1.8.2
```
