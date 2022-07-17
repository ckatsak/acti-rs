# acti-rs

[![GitHub License](https://img.shields.io/github/license/ckatsak/acti-rs)](LICENSE)
[![deps.rs](https://deps.rs/repo/github/ckatsak/acti-rs/status.svg)](https://deps.rs/repo/github/ckatsak/acti-rs)

## Build

### Images

To build the OCI image (it's only the `acti-registrant` for now):

```console
$ make OWNER=... image
```

To push it to the local registry:

```console
$ make OWNER=... LOCAL_REGISTRY=... push-local
```

and to the public registry:

```console
$ make OWNER=... PUBLIC_REGISTRY=... push-public
```

### Local toolchain

Note that building all crates in the workspace requires `hwloc-2.7.1` to be
installed and reachable, due to the `libhwloc2-rs` dependency.

```console
$ cargo build --release
```

### Utilities

#### `crdgen`

Executable that prints to stdout the `CustomResourceDefinition` Kubernetes API
Objects defined in the `acticrds` crate in YAML format, allowing to easily
define them.

For a containerized build of the executable (stored locally):

```console
$ make crdgen
```

To build it, run it and dump the CRDs into a file:

```console
$ make generate-yaml-crds
```
