# SDF Repository

This directory contains the SDF Repository, a web service that offers a REST API for registering, updating, deleting, and listing SDF models.

To reconfigure the SDF Repository, you need to create a `cfg.toml` file from the example and adjust the parameters that are included there.

A current version of the SDF Repository is running under https://sdf-repository.org.
Note that there is currently no real werb interface, so you currently need to interact with the repository using the @SDF Manager.

## Running the SDF Repository

To start the application, just invoke

```sh
cargo run
```
