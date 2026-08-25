# Build all example models
examples:
    #!/usr/bin/env bash
    set -euo pipefail
    for example in examples/*.rs; do
        name="$(basename "${example}" .rs)"
        cargo run --example "${name}"
    done
