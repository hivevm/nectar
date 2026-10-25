#!/usr/bin/env bash
# Installs the system packages and tools of the dev container.
set -euo pipefail

# The GitHub CLI pushes and opens pull requests; it comes from GitHub's own package repository.
GH_KEYRING=/etc/apt/keyrings/githubcli-archive-keyring.gpg
sudo mkdir -p -m 755 /etc/apt/keyrings
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo tee "$GH_KEYRING" >/dev/null
sudo chmod go+r "$GH_KEYRING"
echo "deb [arch=$(dpkg --print-architecture) signed-by=$GH_KEYRING] https://cli.github.com/packages stable main" \
  | sudo tee /etc/apt/sources.list.d/github-cli.list >/dev/null

# PlantUML needs font metrics from headless AWT.
# Graphviz serves the `dot` engine of Nectar.
# Poppler renders PDF pages to images, to look at what the PDF renderer produced.
sudo apt-get update
sudo apt-get install -y --no-install-recommends fontconfig libfreetype6 graphviz poppler-utils gh
sudo rm -rf /var/lib/apt/lists/*

# PlantUML serves the `uml`, `ebnf`, `regex` and `jsongraph` engines of Nectar, pinned by checksum.
PLANTUML_VERSION=1.2025.9
PLANTUML_SHA256=9bdd860a760f082090bcda37a7c4d2c4e02ddf1ada04c3f884030727f4b87773
PLANTUML_JAR=/usr/local/lib/plantuml/plantuml.jar

download=$(mktemp)
curl -fsSL -o "$download" \
  "https://repo1.maven.org/maven2/net/sourceforge/plantuml/plantuml-mit/${PLANTUML_VERSION}/plantuml-mit-${PLANTUML_VERSION}.jar"
echo "${PLANTUML_SHA256}  ${download}" | sha256sum -c -
sudo install -D -m 644 "$download" "$PLANTUML_JAR"
rm -f "$download"

sudo tee /usr/local/bin/plantuml >/dev/null <<EOF
#!/bin/sh
exec java -Djava.awt.headless=true -jar ${PLANTUML_JAR} "\$@"
EOF
sudo chmod 755 /usr/local/bin/plantuml
