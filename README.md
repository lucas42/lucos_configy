# lucos_configy
Configuration Management System for the LucOS ecosystem


## HTTP Endpoints

* `/systems` - Lists all systems.
* `/systems/subdomain/{root_domain}` - Lists systems whose domain ends with the given {root_domain}.
* `/systems/http` - Lists systems which have a `http_port`.
* `/systems/host/{host}` - Lists systems whose `hosts` list contains the given {host}.
* `/volumes` - Lists all volumes.
* `/hosts` - Lists all hosts.