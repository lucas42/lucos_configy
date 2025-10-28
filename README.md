# lucos_configy
Configuration Management System for the LucOS ecosystem


## HTTP Endpoints

* `/systems` - Lists all systems.
* `/systems/subdomain/{root_domain}` - Lists systems whose domain ends with the given {root_domain}.
* `/systems/http` - Lists systems which have a `http_port`.
* `/systems/host/{host}` - Lists systems whose `hosts` list contains the given {host}.
* `/volumes` - Lists all volumes.
* `/hosts` - Lists all hosts.
* `/hosts/http` - Lists hosts which serve http.
* `/components` - Lists all components.

## Available formats
Endpoints support the following formats, using standard content negotiation based on the request's `Accept` header:
* `application/json` - JSON (default).
* `application/x-yaml` - YAML.
* `text/csv;header=present` - Comma-separated values, where the first row specifies the variable names.
* `text/csv;header=absent` - Comma-separated values, where there is no header row.

## Query parameters
The following GET parameters can be added to the endpoints to control the output:
* `fields` - A comma-separated list of field names to include in the response (defaults to all fields)