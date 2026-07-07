#!/usr/bin/env bash
set -euo pipefail

repo="iancleary/montycarlo"
default_branch="main"

dry_run=0
print_current=0
print_next=0
version=""
notes_file=""

usage() {
  cat <<'USAGE'
Usage:
  scripts/cut-release.sh --print-current-version
  scripts/cut-release.sh --print-next-version --version <semver>
  scripts/cut-release.sh --dry-run --version <semver> --notes-file <path>
  scripts/cut-release.sh --version <semver> --notes-file <path>

This repo does not have enough evidence to infer the next release version.
Pass --version explicitly for next-version queries, dry runs, and real releases.
USAGE
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      dry_run=1
      shift
      ;;
    --print-current-version)
      print_current=1
      shift
      ;;
    --print-next-version)
      print_next=1
      shift
      ;;
    --version)
      [[ $# -ge 2 ]] || die "--version requires a value"
      version="$2"
      shift 2
      ;;
    --notes-file)
      [[ $# -ge 2 ]] || die "--notes-file requires a path"
      notes_file="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown argument: $1"
      ;;
  esac
done

repo_root="$(git rev-parse --show-toplevel 2>/dev/null)" || die "run from inside a git checkout"
cd "$repo_root"

current_version() {
  cargo metadata --no-deps --format-version 1 |
    python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])'
}

validate_version() {
  local value="$1"
  [[ "$value" =~ ^[0-9]+\.[0-9]+\.[0-9]+([+-][0-9A-Za-z.-]+)?$ ]] ||
    die "--version must be a SemVer value such as 0.2.0"
}

require_clean_tree() {
  [[ -z "$(git status --porcelain)" ]] || die "working tree must be clean"
}

require_main_branch() {
  local branch
  branch="$(git branch --show-current)"
  [[ "$branch" == "$default_branch" ]] ||
    die "real releases must run from $default_branch; current branch is $branch"
}

require_notes_file() {
  [[ -n "$notes_file" ]] || die "--notes-file is required"
  [[ -s "$notes_file" ]] || die "--notes-file must point to a non-empty file"
}

require_no_existing_tag() {
  local tag="$1"
  git rev-parse -q --verify "refs/tags/$tag" >/dev/null &&
    die "local tag already exists: $tag"
  git ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null 2>&1 &&
    die "remote tag already exists: $tag"
  return 0
}

update_version_files() {
  local next="$1"
  python3 - "$next" <<'PY'
from pathlib import Path
import re
import sys

version = sys.argv[1]

cargo = Path("Cargo.toml")
cargo_text = cargo.read_text()
cargo_text, cargo_count = re.subn(
    r'(?m)^version = "[^"]+"$',
    f'version = "{version}"',
    cargo_text,
    count=1,
)
if cargo_count != 1:
    raise SystemExit("expected one package version in Cargo.toml")
cargo.write_text(cargo_text)

readme = Path("README.md")
readme_text = readme.read_text()
readme_text, readme_count = re.subn(
    r'(?m)^montycarlo = "[^"]+"$',
    f'montycarlo = "{version}"',
    readme_text,
    count=1,
)
if readme_count != 1:
    raise SystemExit("expected one README dependency version")
readme.write_text(readme_text)
PY
  cargo metadata --no-deps --format-version 1 >/dev/null
}

run_checks() {
  cargo fmt --all --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test
  cargo build --release
  cargo package --allow-dirty
}

run_dry_run() {
  local next="$1"
  local tag="v$next"
  local tmp
  tmp="$(mktemp -d)"
  trap "rm -rf '$tmp'" EXIT

  git archive --format=tar HEAD | tar -x -C "$tmp"
  cd "$tmp"
  update_version_files "$next"
  run_checks

  printf 'Dry run complete for %s.\n' "$tag"
  printf 'Would update Cargo.toml, Cargo.lock, and README.md.\n'
  printf 'Would commit, tag, push %s, push %s, then create a GitHub release.\n' "$default_branch" "$tag"
}

if [[ "$print_current" -eq 1 ]]; then
  [[ "$print_next" -eq 0 && "$dry_run" -eq 0 && -z "$version" && -z "$notes_file" ]] ||
    die "--print-current-version cannot be combined with release options"
  current_version
  exit 0
fi

if [[ "$print_next" -eq 1 ]]; then
  [[ -n "$version" ]] || die "cannot infer the next version safely; pass --version"
  validate_version "$version"
  printf '%s\n' "$version"
  exit 0
fi

[[ -n "$version" ]] || die "--version is required because next-version inference is not safe for this repo"
validate_version "$version"
require_notes_file
require_clean_tree

current="$(current_version)"
[[ "$version" != "$current" ]] || die "new version must differ from current version $current"
tag="v$version"
require_no_existing_tag "$tag"

if [[ "$dry_run" -eq 1 ]]; then
  run_dry_run "$version"
  exit 0
fi

require_main_branch
update_version_files "$version"
run_checks

git add Cargo.toml Cargo.lock README.md
git commit -m "chore: release $tag"
git tag -a "$tag" -F "$notes_file"
git push origin "HEAD:$default_branch"
git push origin "$tag"
gh release create "$tag" --repo "$repo" --title "$tag" --notes-file "$notes_file"
