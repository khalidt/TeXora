#!/bin/sh
# Remakes the preview pictures of the templates in assets/templates/ after a template changes.
# Each template is compiled in a temporary folder, then TeXora draws its title slide (<id>-1.png)
# and its "Key Ideas" slide (<id>-5.png) at 400 pixels wide. Needs pdflatex and a built TeXora.
# Documents (not slides) show their first two pages instead, in a page-shaped window.
# Usage: sh scripts/make-template-previews.sh [id ...]   (no id: every template)
set -eu
cd "$(dirname "$0")/.."
[ -f .toolchain/env.sh ] && . ./.toolchain/env.sh
cargo build --quiet
BIN=./target/debug/TeXora
WORK=$(mktemp -d)
# The animated templates need their pictures and clip beside them.
cp -R assets/templates/media "$WORK/media"
cp assets/templates/video/* "$WORK/media/"
cp assets/templates/media/frame-1.png "$WORK/media/test-card.png"
cp assets/tex/*.sty "$WORK/"
if [ $# -gt 0 ]; then list=""; for id in "$@"; do list="$list assets/templates/$id.tex"; done; else list=$(ls assets/templates/*.tex); fi
for tex in $list; do
  id=$(basename "$tex" .tex)
  if grep -q 'usepackage{texoravideo}' "$tex"; then
    # Its clips need posters, which TeXora makes when it compiles: create the deck with TeXora itself.
    echo '{"check_updates":false}' > "$WORK/settings.json"
    TEXORA_CONFIG="$WORK/settings.json" "$BIN" --new "$WORK/$id.tex" --template "$id" --view studio --window 1388x900 --script "w:12000" --screenshot "$WORK/$id-compile.png" >/dev/null
  else
    cp "$tex" "$WORK/$id.tex"
    (cd "$WORK" && pdflatex -interaction=nonstopmode "$id.tex" >/dev/null; pdflatex -interaction=nonstopmode "$id.tex" >/dev/null) || true
  fi
  # The window matches the slide or page shape, so the picture has no bars.
  pages="0:1 4:5"
  if grep -q 'documentclass\[11pt\]{article}' "$tex"; then size=630x815; pages="0:1 1:5"
  elif grep -q 'aspectratio=43' "$tex"; then size=840x630; else size=1120x630; fi
  for pair in $pages; do
    page=${pair%:*}; name=${pair#*:}
    echo '{"check_updates":false}' > "$WORK/settings.json"
    TEXORA_CONFIG="$WORK/settings.json" "$BIN" "$WORK/$id.pdf" --view audience --goto "$page" --window "$size" --screenshot "$WORK/$id-$name.png" >/dev/null
    if [ "$size" = 630x815 ]; then
      # The window may be capped by the screen: keep only the page (US Letter, 8.5 by 11).
      h=$(sips -g pixelHeight "$WORK/$id-$name.png" | awk '/pixelHeight/{print $2}')
      sips --cropToHeightWidth "$h" $((h * 85 / 110)) "$WORK/$id-$name.png" >/dev/null
      sips --resampleWidth 400 "$WORK/$id-$name.png" --out "assets/templates/$id-$name.png" >/dev/null
    else
      sips -Z 400 "$WORK/$id-$name.png" --out "assets/templates/$id-$name.png" >/dev/null
    fi
  done
  echo "Previews made for $id"
done
