# Markdown  *[Link](https://www.google.com)* ~~generation~~ ```adljaslkdj``` asdashdk



The plugin supports [for][id] the creation of beautiful PDF's from markdown. Create a simple gradle project with the Gradle
Wrapper and configure *build.gradle* and *settings.gradle*.

The *build.gradle* applies the documentation plugin with the configuration parameters.

~~~gradle
plugins {
  id 'org.hivevm.doc' version '1.1.1'
}

markdown {
  source   = 'manual'
  template = 'template.ui.xml'
}
~~~

The latest version can always be loaded from [Gradle Plugin repository](https://plugins.gradle.org/plugin/org.hivevm.doc).

The optional *markdown.config* parameter defines the configuration file. If the parameter is omitted an internal default
template is used. The *markdown.source* parameter defines a directory of where the root markdown files are located, or
it defines a single markdown file that should be converted to PDF.

The *settings.gradle* optionally defines the plugin repository.

~~~gradle
pluginManagement {
  repositories {
    gradlePluginPortal()
  }
}
~~~

The *markdown.config* parameter is optional and allows custom configuration of the layout of the rendered PDF. By
default a standard template is applied.

## Common mark

[Markdown][1] is a plain text format for writing structured documents, based on formatting conventions from email and
usenet. *Markdown* is a simple way to format text that looks great on any device. It doesn’t do anything fancy like
change the font size, color, or type — just the essentials, using keyboard symbols you already know.

There are different implementations of Markdown, we use the [Common Markdown][2] is intended to be as easy-to-read and
easy-to-write as is feasible.

Readability, however, is emphasized above all else. A Markdown-formatted document should be publishable as-is, as plain
text, without looking like it’s been marked up with tags or formatting instructions. While Markdown’s syntax has been
influenced by several existing text-to-HTML filters — including Setext, atx, Textile, reStructuredText, Grutatext, and
EtText — the single biggest source of inspiration for Markdown’s syntax is the format of plain text email.

To this end, Markdown's syntax is composed entirely of punctuation characters, which punctuation characters have been
carefully chosen so as to look like what they mean. E.g., asterisks around a word actually look like *emphasis*.
Markdown lists look like, well, lists. Even blockquotes look like quoted passages of text, assuming you’ve ever used
email.

Common mark is a strongly defined, highly compatible specification of Markdown.

For details look the [Common Mark Manual](https://commonmark.org/help/). There are many ways to write markdown; a simple
offline editor is [Typora][3].

[1]: https://en.wikipedia.org/wiki/Markdown "Markdown is a lightweight markup language for creating formatted text using a plain-text editor."

[2]: https://spec.commonmark.org/ "A strongly defined, highly compatible specification of Markdown"

[3]: https://typora.io/ "A truly minimal Mardkown editor"

### Headings

Starting a line with a hash # and a space makes a header.

The more #, the smaller the header.

	# Manual title
	## Chapter
	### Sub Chapter 

!w Remember: Each *markdown* file should be handled as independent document. Including a file into another it becomes
automatically a child chapter. So use the highest heading in each file.

### Paragraphs

A paragraph is consecutive lines of text with one or more blank lines between them.

	lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor
	invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.

### Line Breaks

For a line break, add either a backslash \ or two blank spaces at the end of the line; this is interpreted as
continuation of the text. This is useful if the markdown text should be be wrapped for better reading, but logically the
text remains on the same line.

	lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy··
	eirmod tempor
or

	lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy\
	eirmod tempor

lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy\
eirmod tempor

A text with less than two spaces is interpreted as simple line break

	lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy 
	eirmod tempor

lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy
eirmod tempor

To organize the text thematic 3 sings(-), underscores(_), or stars(*) can be used. On PDF this will create a page break.

	lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy 
	___
	eirmod tempor

lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy
___
eirmod tempor

### Blockquotes

To create a blockquote, start a line with greater than > followed by an optional space.

Blockquotes can be nested, and can also contain other formatting.

	> First line
	> Another line
	>
	> > Nested line
	>
	> Last line	

> First line
> Another line
>
> > Nested line
>
> Last line

### Emphasis

To create bold or italic, wrap with asterisks \* or underscores \_. To avoid creating bold or italic, place a backslash
in front \\\* or \\\_. For *italics* a single asteriks or underscore is used, for **bold** 2 are used. Using 3 allows to
create a ***bold and italic*** text.

Our implementation supports an additional syntax for ____underline____ with 4 underscores and ~~strikethrough~~ with 2
tilde.

	*italic* _italic_
	**bold** __bold__
	***bold & italic text***
	~~strikethrough~~
	____underline____

### Lists

Unordered lists can use either asterisks *, plus +, or hyphens - as list markers.

	- Apples
	- Oranges
	- Pears

- Apples
- Oranges
- Pears

Ordered lists use numbers followed by period . or right paren ).

	1. First
	2. Second
	3. Third

1. First
2. Second
3. Third



[id]: http://b.org/ "text"