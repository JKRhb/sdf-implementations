# SDF Proof-of-Concept Implementations

This repository contains a set of proof of concept implementations that have been created in the context of my Master's Thesis on derivation and instantiation in SDF.

The implementations include an [`SDF Repository`](./sdf-repository/) for hosting SDF models, an [`SDF Manager`](./sdf-manager/) for adminstrating the SDF Repository, and an SDF Consumer [`SDF Consumer`](/sdf-consumer/). for interacting with an [`SDF Thing`](/sdf-thing/).
Lastly, a common [`SDF Data Structures`](/sdf-data-structures/) crate contains reusable definitions share by all binary crates.

## Running the Applications

After installing a current Rust version, you can run the three workspace member applications (`sdf-manager`, `sdf-repository`, `sdf-consumer`) by specifying their name with the `bin` argument of `cargo run` like so:

```sh
    cargo run --bin sdf-repository
```

In practice, it probably makes more sense to `cd` into the application directory and execute `cargo run` from there.
