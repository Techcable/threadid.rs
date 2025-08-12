
format: && spellcheck
    taplo format
    cargo +nightly fmt

check: check-format
    -just spellcheck
    cargo clippy
    cargo doc --no-deps
    cargo rdme --check
    lychee README.md

check-format: && spellcheck
    taplo format
    cargo +nightly fmt --check

spellcheck:
    typos
    git log | typos -

fix-spelling:
    typos --write-changes
    git log | typos -
