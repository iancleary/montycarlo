# Release Process

Use the portable `create-release-process` skill when maintaining this workflow.
Use the repo-local `cut-release` workflow for ordinary releases.

## Versioning

The release version source is the package `version` in `Cargo.toml`. This repo
uses SemVer tags of the form `vMAJOR.MINOR.PATCH`; the existing tag history does
not define a safe next-version inference policy, so releases must pass
`--version` explicitly.

Read-only queries:

```sh
just cut-release --print-current-version
just cut-release --print-next-version --version 0.2.0
```

The next-version query prints the explicit `--version` value. Without
`--version`, it fails instead of guessing.

## Release Notes

Prepare a non-empty Markdown notes file before release and pass it with
`--notes-file`. The notes file is used for the annotated git tag and the GitHub
release body.

## Dry Run

Run a dry run before a real release:

```sh
just cut-release --dry-run --version 0.2.0 --notes-file /path/to/notes.md
```

Dry runs operate on a temporary archive of `HEAD`, run validation, and do not
push commits, create tags, publish crates, or create GitHub releases.

## Real Release

Real releases must run from a clean `main` branch:

```sh
just cut-release --version 0.2.0 --notes-file /path/to/notes.md
```

The runner owns these versioned files:

- `Cargo.toml`
- `Cargo.lock`
- `README.md`

The runner validates with:

- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo build --release`
- `cargo package --allow-dirty`

The real release path commits the version bump, creates an annotated `vX.Y.Z`
tag, pushes `main`, pushes the tag, and then creates the GitHub release with
`gh release create`. The final public-facing action is GitHub release creation.
This workflow does not publish to crates.io; add that only after the repo has an
explicit crates.io publishing policy.
