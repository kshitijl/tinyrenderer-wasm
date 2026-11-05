#!/bin/sh
# Builds the page and copies web/ to the site's public/demos/tinyrenderer-rs/ (the site never
# edits its copy). SITE defaults to ~/local-src/39-klauria-tech.
set -e
cd "$(dirname "$0")/.."
SITE=${SITE:-$HOME/local-src/39-klauria-tech}
rm -rf web/pkg
just build
rsync -a --delete web/ "$SITE/public/demos/tinyrenderer-rs/"
echo "synced to $SITE/public/demos/tinyrenderer-rs/ (commit it there)"
