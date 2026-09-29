# lucos_configy
Configuration Management System for the LucOS ecosystem


## Using the API

### HTTP Endpoints

* `/systems` - Lists all systems.
* `/systems/subdomain/{root_domain}` - Lists systems whose domain ends with the given {root_domain}.
* `/systems/http` - Lists systems which have a `http_port`.
* `/systems/host/{host}` - Lists systems whose `hosts` list contains the given {host}.
* `/systems/host/{host}/public-ports` - Returns a flat list of `{system, port, protocol, purpose}` records for all public ports declared on systems whose `hosts` list contains the given {host}. Intended for consumption by the firewall generator.
* `/volumes` - Lists all volumes.
* `/hosts` - Lists all hosts.
* `/hosts/http` - Lists hosts which serve http.
* `/components` - Lists all components.
* `/scripts` - Lists all scripts.
* `/repositories/{id}` - Returns a single repository (system, component, or script) by its id. Searches across all three types and includes a `type` field (`"system"`, `"component"`, or `"script"`) in the response. Returns 404 if no repository with the given id is found. Note: this endpoint does not support CSV format (returns JSON or YAML only).

### Available formats
Endpoints support the following formats, using standard content negotiation based on the request's `Accept` header:
* `application/json` - JSON (default).
* `application/x-yaml` - YAML.
* `text/csv;header=present` - Comma-separated values, where the first row specifies the variable names.
* `text/csv;header=absent` - Comma-separated values, where there is no header row.

### Query parameters
The following GET parameters can be added to the endpoints to control the output:
* `fields` - A comma-separated list of field names to include in the response (defaults to all fields)

### Reading optional fields

Optional fields appear in every response, even when absent in the underlying YAML — they are serialised as an explicit `null`, not omitted from the response. For example, a host without a `backup_root` set in its YAML still has a `backup_root` key in the JSON output, with the value `null`.

This trips up the most natural reader idiom in several languages — `dict.get(key, default)` and friends only fall back when the key is **absent**, not when it is present with a null value. Use a null-coalescing idiom instead:

**Python:**
```python
# WRONG — only falls back when the key is absent.
# Returns None when the key is present with a null value.
backup_root = host.get('backup_root', '/')

# RIGHT — falls back on null and missing alike.
backup_root = host.get('backup_root') or '/'
```

**Go** (decoding into `map[string]interface{}`):
```go
// WRONG — `ok` is true when the key is present, including when the value is null.
// The fallback is skipped and v is left as nil.
v, ok := host["backup_root"]
if !ok {
    v = "/"
}
// v is nil here when the JSON had {"backup_root": null}

// RIGHT — type-assert, and treat both "null/wrong type" and "missing" the same way.
backupRoot, _ := host["backup_root"].(string)
if backupRoot == "" {
    backupRoot = "/"
}
```

(Decoding into a typed struct does not distinguish absent from null either way — both surface as the zero value of the field's type.)

The same shape applies to other languages with similar idioms (e.g. Java/Kotlin `Optional.orElse`, Ruby `Hash#fetch`).

When testing consumers, exercise them against the live configy API or a fixture that mirrors its serialisation (every key present, with `null` for absent values). A YAML-only fixture where the key is omitted does **not** match the API's behaviour and will hide this class of bug — see the [2026-04-28 lucos_backups Aurora cron incident](https://github.com/lucas42/lucos/blob/main/docs/incidents/2026-04-28-backups-aurora-null-config-cron-failure.md) for an example of how this fails in practice.


## System fields

| Field | Type | Description |
|---|---|---|
| `domain` | string (optional) | Public-facing domain for this system. |
| `http_port` | integer (optional) | Internal port on which this system serves HTTP traffic (router backend). |
| `hosts` | list of strings | Host(s) this system runs on. |
| `unsupervisedAgentCode` | boolean (default: false) | Whether this system's code may be changed by AI agents without human review. |
| `public_ports` | list of port entries (default: []) | Ports that are publicly reachable on this system's host(s). Used by `lucos_firewall` to generate iptables rules. See below. |

### `public_ports` entries

Each entry in `public_ports` has three required fields:

| Field | Type | Description |
|---|---|---|
| `port` | integer (1–65535) | Port number. |
| `protocol` | string: `tcp` or `udp` | Network protocol. Invalid values cause config load to fail. |
| `purpose` | string | Free-form human-readable description of what this port is used for. |

Example:

```yaml
lucos_mail:
    domain: mail.l42.eu
    http_port: 8022
    hosts: [avalon]
    public_ports:
        - { port: 25, protocol: tcp, purpose: "SMTP inbound" }
        - { port: 587, protocol: tcp, purpose: "SMTP submission" }
```

## Volume fields

| Field | Type | Description |
|---|---|---|
| `description` | string (optional) | Human-readable description of what the volume holds. |
| `recreate_effort` | string (optional) | How hard the data is to recreate if lost (e.g. `small`, `considerable`, `huge`, `automatic`). Recognised values are validated against `lucos_backups`. |
| `skip_backup` | boolean (default: false) | When true, `lucos_backups` does not back this volume up at all. |
| `skip_backup_on_hosts` | list of strings (default: []) | Hosts to exclude as backup *destinations* for this volume. |
| `backup_strategy` | string (default: `full-snapshot`) | Backup mechanism `lucos_backups` uses for this volume: `full-snapshot` (daily full tar+scp) or `incremental` (rsync `--link-dest` hardlink-rotated snapshots, for large append-mostly media volumes). See ADR-0002 in `lucos_backups`. |
| `pause_during_backup` | boolean (default: false) | When true, `lucos_backups` pauses the containers writing to this volume while it makes its fast local copy, so the copy is a single point in time rather than a smear across a live write. Set it on volumes mutated in place by a running writer (databases); leave it off for append-only media. Not supported with `backup_strategy: incremental`. See ADR-0002 in `lucos_backups`. |

## Updating the data
Edit YAML files in the `config` directory.
Commit the change to the main branch and push to github.
The updated API will be automatically deployed.

The API is live once configy's own deploy restarts it, a few minutes after the merge. Its consumers then each pick up the change on their own schedule, and some need a manual step. For a **new system**, do these in order:

1. **Credentials, before the system's first deploy.** `lucos_creds`' configy sync writes `PORT` and `APP_ORIGIN` for `development` and `production`, but only hourly at :53. A deploy that runs first gets an empty `PORT` and fails. Wait for the :53 run, or have lucas42 set both in production by hand. A `Credential PORT updated in <system> (production)` event in loganne confirms it.
2. **DNS: automatic.** `lucos_dns` syncs every 15 minutes and adds `<domain>` as a CNAME to `<host>.s.l42.eu`.
3. **Router and TLS: manual, once DNS resolves.** `lucos_router` reads configy only when it starts and at 22:16 UTC daily. Until then the domain gets the host's default certificate. Run `docker exec lucos_router update-domains.sh` on each host listed in `hosts`. That issues the certificate and reloads nginx, with no restart.
4. **Monitoring: manual rebuild, once `/_info` serves.** `lucos_monitoring` takes its list of systems from configy at image build time, so restarting it does nothing. Trigger a `main` pipeline for lucos_monitoring. Do this only once the system's `/_info` answers, or it goes straight to red.
5. **Automatic, no action needed:** `lucos_backups` re-reads volumes hourly at :03. Its `volume-host` check fails until the new volumes exist on the host. `lucos_root` rereads configy every 5 minutes and adds the homepage tile once `/_info` answers. The code-reviewer auto-merge workflow reads `unsupervisedAgentCode` per PR, and `lucos_repos` reads configy at the start of each 6-hourly sweep. `lucos_firewall` reads `public_ports`.

## Running tests
Tests are located in the `api` directory.

### API Logic Tests
These tests validate the application logic using mock data. They do not depend on the actual contents of the `config` directory.
Run them using:
```bash
cd api
cargo test --test api_logic
```

### Config Validation
This validates that the YAML files in the `config` directory are valid and match the application's data models.
Run them using:
```bash
cd api
cargo test --test validation
```

### All Tests
To run both sets of tests (and all other unit tests):
```bash
cd api
cargo test
```