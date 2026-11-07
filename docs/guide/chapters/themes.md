# Themes

A theme is a directory `themes/<name>/` with the [minijinja](https://docs.rs/minijinja)
templates `layouts/page.html` for the website and `layouts/print.html` for the print view, and
`static/`, whose files are copied to `_theme/` in the output. A project theme overrides the
built-in theme file by file.
