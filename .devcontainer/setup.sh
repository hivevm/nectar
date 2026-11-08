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
# strace traces a build in the Q-5 test.
sudo apt-get update
sudo apt-get install -y --no-install-recommends fontconfig libfreetype6 graphviz poppler-utils strace gh
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

# cargo-deny checks the licences and the crates of the workspace (ADR-0008); the release archive
# for the container's architecture, pinned by checksum.
CARGO_DENY_VERSION=0.20.2
case "$(uname -m)" in
  x86_64) CARGO_DENY_ARCH=x86_64 CARGO_DENY_SHA256=9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f ;;
  aarch64) CARGO_DENY_ARCH=aarch64 CARGO_DENY_SHA256=995c82be0defc7a025cae49a2aa2644ce8245c9a3318fc4103907c6a285e8c7d ;;
  *) echo "setup.sh: no cargo-deny archive for $(uname -m)" >&2; exit 1 ;;
esac
CARGO_DENY_NAME="cargo-deny-${CARGO_DENY_VERSION}-${CARGO_DENY_ARCH}-unknown-linux-musl"

download=$(mktemp)
curl -fsSL -o "$download" \
  "https://github.com/EmbarkStudios/cargo-deny/releases/download/${CARGO_DENY_VERSION}/${CARGO_DENY_NAME}.tar.gz"
echo "${CARGO_DENY_SHA256}  ${download}" | sha256sum -c -
unpacked=$(mktemp -d)
tar -xzf "$download" -C "$unpacked"
sudo install -m 755 "$unpacked/$CARGO_DENY_NAME/cargo-deny" /usr/local/bin/cargo-deny
rm -rf "$download" "$unpacked"
