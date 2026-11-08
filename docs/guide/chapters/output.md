# Output

`nectar build` writes, and `nectar serve` serves, for a project; `nectar build <file>` writes only
the PDF of that file ([Quick start](quick-start.md)):

```
public/
  index.html                 site index, when every document has a directory of its own
  <document>/index.html      root chapter
  <document>/<slug>/index.html
  <document>/<asset path>    local images and files, relative to the start document
  <document>/_remote/        images from the web
  <document>/print/          print view and PDF
  <document>/epub/           the EPUB, when the document enables it
  _graphics/<hash>.svg       diagrams, named by their content
  _theme/                    the theme's static files
```

Links between the files are relative, so the output also works when opened from disk.
