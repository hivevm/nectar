# Quick start

A project is a directory with a `nectar.toml`:

```toml
title = "My Manual"

[[document]]
source = "content/manual.md"
```

```sh
nectar serve            # http://127.0.0.1:1313/, reloads on every change
nectar build            # writes the site to public/
```

Run either command in the project directory, or name the directory with `-C <dir>`.
This guide is such a project itself: `docs/guide` in the repository is a complete project to
start from.

| Option | Meaning |
|---|---|
| `-C <dir>` | Project directory, the one that holds `nectar.toml`; default the current one |
| `--offline` | Keep images from the web as links instead of downloading them |
| `build -o <dir>` | Output directory; overrides `output` in `nectar.toml` |
| `build --drafts` | Include files whose front matter says `draft = true` |
| `serve -p <port>`, `serve --bind <addr>` | Address to serve on; default `127.0.0.1:1313` |
| `serve --no-drafts` | Leave out drafts, which `serve` shows by default |
