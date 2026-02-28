#!/usr/bin/env bash
set -euo pipefail

# docs_build.sh
# - Builds the Zensical site from mkdocs.yml
# - Builds rustdoc (public API) and merges it under site/api/
#
# CI usage:
#   ZENSICAL_NO_VENV=1 ./scripts/docs_build.sh
# (expects `zensical` already installed in PATH)

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

# Clean site output deterministically
rm -rf site

if [[ "${ZENSICAL_NO_VENV:-}" != "1" ]]; then
  if [[ ! -d .venv ]]; then
    python -m venv .venv
  fi
  # shellcheck disable=SC1091
  source .venv/bin/activate
  python -m pip install --upgrade pip
  pip install zensical
fi

# Build docs site
zensical build

# Build rustdoc and merge into the generated site.
# - --no-deps keeps the published API docs focused and small.
# - --all-features ensures feature-gated items are documented.
cargo doc --no-deps --all-features

rm -rf site/api
mkdir -p site/api
cp -R target/doc/* site/api/

# Custom domain for GitHub Pages
printf '%s\n' "termgrid.entropy.quest" > site/CNAME
