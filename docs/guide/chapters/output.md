# Output

`nectar build` writes, and `nectar serve` serves:

```
public/
  index.html                 site index, when every document has a directory of its own
  <document>/index.html      root chapter
  <document>/<slug>/index.html
  <document>/<asset path>    local images and files, relative to the start document
  <document>/_remote/        images from the web
  <document>/print/          print view and PDF
  _graphics/<hash>.svg       diagrams, named by their content
  _theme/                    the theme's static files
```

Links between the files are relative, so the output also works when opened from disk.
