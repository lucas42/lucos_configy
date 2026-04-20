# lucos_configy API — notes for Claude Code

## Running tests

Tests require the `config/` directory from the repo root to be available at `api/config/`. When running via Docker:

```bash
docker run --rm -v "$PWD/api:/work" -v "$PWD/config:/work/config" -w /work rust:1.95.0-alpine3.22 \
  sh -c "apk add --no-cache musl-dev pkgconfig && cargo test"
```

## RDF export (`api/src/all.rs`)

The `GET /all` turtle endpoint serializes all five entity types (System, Host, Volume, Component, Script) and the configy ontology.

**Keep `all.rs` in sync with `data.rs`.** When a new field is added to any struct in `data.rs`, or a new entity type is added, the corresponding `turtle_*` function in `all.rs` must be updated to include it. The compiler won't catch missing fields in the hand-written serializer.

The configy ontology namespace (`configy:`) is derived from the `APP_ORIGIN` environment variable at request time, so URI prefixes are correct across environments.

Entity URIs use fragments (`/systems#id`, `/hosts#id`, etc.) rather than paths, since per-item path endpoints don't exist.
