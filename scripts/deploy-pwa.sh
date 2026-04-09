#!/bin/bash
set -euo pipefail

# Deploy the PWA build to the gh-pages branch for GitHub Pages hosting.
# Requires: gh (authenticated), bun, wasm-pack (via cargo)

REPO_NAME=$(gh repo view --json name -q .name)
BASE_PATH="/$REPO_NAME"

echo "Building PWA for $REPO_NAME (base: $BASE_PATH)..."

# Build
cd app
bun run build:wasm
PUBLIC_BASE_PATH="$BASE_PATH" bunx vite build
cd ..

echo "Build complete. Deploying to gh-pages branch..."

# Deploy via git worktree
DEPLOY_DIR=$(mktemp -d)
trap 'git worktree remove "$DEPLOY_DIR" --force 2>/dev/null; rm -rf "$DEPLOY_DIR"' EXIT

if git show-ref --quiet refs/remotes/origin/gh-pages; then
    git worktree add "$DEPLOY_DIR" gh-pages
else
    git worktree add --orphan -b gh-pages "$DEPLOY_DIR"
fi

# Copy build output, preserve .nojekyll to allow _ prefixed files on GH Pages
cp -r app/build/. "$DEPLOY_DIR/"
touch "$DEPLOY_DIR/.nojekyll"

cd "$DEPLOY_DIR"
git add -A

if git diff --staged --quiet; then
    echo "Nothing changed, skipping deploy."
    exit 0
fi

COMMIT_SHA=$(git -C "$OLDPWD" rev-parse --short HEAD)
git commit -m "deploy: pwa from $COMMIT_SHA"
git push origin gh-pages --force

echo ""
echo "Deployed! Enable GitHub Pages if not already:"
echo "  gh api repos/$(gh repo view --json nameWithOwner -q .nameWithOwner)/pages -X POST -f source[branch]=gh-pages -f source[path]=/"
echo ""
echo "PWA URL: https://$(gh repo view --json owner -q .owner.login).github.io/$REPO_NAME/"
