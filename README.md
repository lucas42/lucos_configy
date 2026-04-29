# lucos_configy
Configuration Management System for the LucOS ecosystem


## Using the API

### HTTP Endpoints

* `/systems` - Lists all systems.
* `/systems/subdomain/{root_domain}` - Lists systems whose domain ends with the given {root_domain}.
* `/systems/http` - Lists systems which have a `http_port`.
* `/systems/host/{host}` - Lists systems whose `hosts` list contains the given {host}.
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


## Updating the data
Edit YAML files in the `config` directory.
Commit the change to the main branch and push to github.
The updated API will be automatically deployed.

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