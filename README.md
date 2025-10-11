# Markdown for PDF generation {#Markdown_for_PDF_generation}

The plugin supports the creation of beautiful PDF's from markdown. Create a simple gradle project with the Gradle
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

Multilevel and combined lists

    1. Fruit
     - Apple
     - Pear
    2. Vegetable
     - Potato
     - Carrot

1. Fruit
   - Apple
     1. Apple 1
     1. Apple 2
   - Pear
     1. Pear 1
     1. Pear 2
2. Vegetable
   - Potato
     1. Potato 1
     1. Potato 2
   - Carrot
     1. [x] Carrot 1
     1. [ ] Carrot 2

Task List

    - [x] Release candidate
    - [ ] Product release

- [x] Release candidate
- [ ] Product release

### Links

Links can be either inline with the text, or placed at the bottom of the text as references.

Link text is enclosed by square brackets [], and for inline links, the link URL is enclosed by parens ().

	[text](http://a.com)

Links can be realized as footnotes, defining an identifier by square brackets [] and the link URL defined below in the
page.

	[][id]
	...
	[id]: http://b.org/ "text"

or as alternative with an inline link

	[text][id]
	...
	[id]: http://b.org/ "text"




[Tests](#Lists)

### Images

Images are almost identical to links, but an image starts with an exclamation point !.

	![](markdown.png)
	![text](markdown.png)

![text](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAeCAQAAADArVVKAAABJWlDQ1BJQ0MgcHJvZmlsZQAAKJGdkD1KxFAUhb/MiH9opVioRQoRLAZsTGUzKgRBIcYRHK0ySQYHkxiSDIM7cCe6mCkEwR24AQVrz4sWFqbxweV8XO49570HLTsJ03JmF9KsKly/27/sX9lzb7TZYIEdtoOwzLued0Lj+XzFMvrSMV7Nc3+e2SguQ+lUlYV5UYG1L3YmVW5Yxeptzz8UP4jtKM0i8ZN4K0ojw2bXT5Nx+ONpbrMUZxfnpq/axOWYUzxsBowZkVDRkWbqHOGwJ3UpCLinJJQmxOpNNFNxIyrl5HIg6ol0m4a89TrPU8pAHiN5mYQ7UnmaPMz/fq99nNWb1to0D4qgbrVVreEQ3h9huQ8rz7B43ZA1//ttDTNOPfPPN34BFKlQexFMbQcAAAACYktHRAD/h4/MvwAAAAlwSFlzAAALEwAACxMBAJqcGAAAAAd0SU1FB+UDGwkqM5WPHG4AAAAZdEVYdENvbW1lbnQAQ3JlYXRlZCB3aXRoIEdJTVBXgQ4XAAACJUlEQVRIx7WWTUiUQRjHf/PuqxWWepMQiXSRLuYlECIvBuJND5UFtgoGdSkEsQ6CVIcgO5id6lCiYHSoWyhFQpB06VCelDYK6cOi2JJ2i9F9dzo4TO375bbu+8xl5vnPzPPM8zkCiwRD1CMoLSneco0p6COLimhk6RUsso/nXCdb4hfYDHCQJXBQHCEKOorCsbBQrEciYB2wLCImO29lUa1nkoxrZzk79SyDzEMq9S0Oq/7hlKNTz3eTRiKRJKl1KXJXI5I+l1Jzmv/SpW4XCmV59NwYcSbZYbiCQboN5j5TpvllfvoH+6CNKwbt4BJFeiv4mOAsCQAamWBbsU4O0yvGOC1UMU1NqaLITZVMs8CB0oXpX0rroGygwXAqiimIQQIm2U/rP+sUI9zIE7CLvdrIFZqznSYcAN7zfTMBGU7yjDqT9Gf47MmfcQ4BwvixnhcoYIH2Qpy8TIJf+qpR7vsYsYd32MTMuwQxbL5wglRhUfSU8+SAh1xG+eAfOcY3F+8nx0kWGqZwkzss0s9aAP6KXn7n1c/TzBeeB+AwQCdfQ3bMMqgdCzkucu9/Em3D2clNOu8txrQBJ7jqNeXW+0GOYR4ATzhn3hKYB5JZzVny7Ewxo9X54ELWOEWaCzrmQvsBCDP8yl8Qgi+3C4WyPeLC7F0EFnlPtpCIrZTjEKoBpOAR7fxgLoKP12GqeQzNrET2dfxEs4jzZg/9NEbw+X3N7fjyHzpu6IUgF9LcAAAAAElFTkSuQmCC)

![](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAeCAQAAADArVVKAAABJWlDQ1BJQ0MgcHJvZmlsZQAAKJGdkD1KxFAUhb/MiH9opVioRQoRLAZsTGUzKgRBIcYRHK0ySQYHkxiSDIM7cCe6mCkEwR24AQVrz4sWFqbxweV8XO49570HLTsJ03JmF9KsKly/27/sX9lzb7TZYIEdtoOwzLued0Lj+XzFMvrSMV7Nc3+e2SguQ+lUlYV5UYG1L3YmVW5Yxeptzz8UP4jtKM0i8ZN4K0ojw2bXT5Nx+ONpbrMUZxfnpq/axOWYUzxsBowZkVDRkWbqHOGwJ3UpCLinJJQmxOpNNFNxIyrl5HIg6ol0m4a89TrPU8pAHiN5mYQ7UnmaPMz/fq99nNWb1to0D4qgbrVVreEQ3h9huQ8rz7B43ZA1//ttDTNOPfPPN34BFKlQexFMbQcAAAACYktHRAD/h4/MvwAAAAlwSFlzAAALEwAACxMBAJqcGAAAAAd0SU1FB+UDGwkqM5WPHG4AAAAZdEVYdENvbW1lbnQAQ3JlYXRlZCB3aXRoIEdJTVBXgQ4XAAACJUlEQVRIx7WWTUiUQRjHf/PuqxWWepMQiXSRLuYlECIvBuJND5UFtgoGdSkEsQ6CVIcgO5id6lCiYHSoWyhFQpB06VCelDYK6cOi2JJ2i9F9dzo4TO375bbu+8xl5vnPzPPM8zkCiwRD1CMoLSneco0p6COLimhk6RUsso/nXCdb4hfYDHCQJXBQHCEKOorCsbBQrEciYB2wLCImO29lUa1nkoxrZzk79SyDzEMq9S0Oq/7hlKNTz3eTRiKRJKl1KXJXI5I+l1Jzmv/SpW4XCmV59NwYcSbZYbiCQboN5j5TpvllfvoH+6CNKwbt4BJFeiv4mOAsCQAamWBbsU4O0yvGOC1UMU1NqaLITZVMs8CB0oXpX0rroGygwXAqiimIQQIm2U/rP+sUI9zIE7CLvdrIFZqznSYcAN7zfTMBGU7yjDqT9Gf47MmfcQ4BwvixnhcoYIH2Qpy8TIJf+qpR7vsYsYd32MTMuwQxbL5wglRhUfSU8+SAh1xG+eAfOcY3F+8nx0kWGqZwkzss0s9aAP6KXn7n1c/TzBeeB+AwQCdfQ3bMMqgdCzkucu9/Em3D2clNOu8txrQBJ7jqNeXW+0GOYR4ATzhn3hKYB5JZzVny7Ewxo9X54ELWOEWaCzrmQvsBCDP8yl8Qgi+3C4WyPeLC7F0EFnlPtpCIrZTjEKoBpOAR7fxgLoKP12GqeQzNrET2dfxEs4jzZg/9NEbw+X3N7fjyHzpu6IUgF9LcAAAAAElFTkSuQmCC){align=float-right}

We support the non-official specification to provide a size and alignment for the image. The optional parameters are
defined between brackets {}.

	![text](markdown.png){width=4cm align=center}

![text](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAeCAQAAADArVVKAAABJWlDQ1BJQ0MgcHJvZmlsZQAAKJGdkD1KxFAUhb/MiH9opVioRQoRLAZsTGUzKgRBIcYRHK0ySQYHkxiSDIM7cCe6mCkEwR24AQVrz4sWFqbxweV8XO49570HLTsJ03JmF9KsKly/27/sX9lzb7TZYIEdtoOwzLued0Lj+XzFMvrSMV7Nc3+e2SguQ+lUlYV5UYG1L3YmVW5Yxeptzz8UP4jtKM0i8ZN4K0ojw2bXT5Nx+ONpbrMUZxfnpq/axOWYUzxsBowZkVDRkWbqHOGwJ3UpCLinJJQmxOpNNFNxIyrl5HIg6ol0m4a89TrPU8pAHiN5mYQ7UnmaPMz/fq99nNWb1to0D4qgbrVVreEQ3h9huQ8rz7B43ZA1//ttDTNOPfPPN34BFKlQexFMbQcAAAACYktHRAD/h4/MvwAAAAlwSFlzAAALEwAACxMBAJqcGAAAAAd0SU1FB+UDGwkqM5WPHG4AAAAZdEVYdENvbW1lbnQAQ3JlYXRlZCB3aXRoIEdJTVBXgQ4XAAACJUlEQVRIx7WWTUiUQRjHf/PuqxWWepMQiXSRLuYlECIvBuJND5UFtgoGdSkEsQ6CVIcgO5id6lCiYHSoWyhFQpB06VCelDYK6cOi2JJ2i9F9dzo4TO375bbu+8xl5vnPzPPM8zkCiwRD1CMoLSneco0p6COLimhk6RUsso/nXCdb4hfYDHCQJXBQHCEKOorCsbBQrEciYB2wLCImO29lUa1nkoxrZzk79SyDzEMq9S0Oq/7hlKNTz3eTRiKRJKl1KXJXI5I+l1Jzmv/SpW4XCmV59NwYcSbZYbiCQboN5j5TpvllfvoH+6CNKwbt4BJFeiv4mOAsCQAamWBbsU4O0yvGOC1UMU1NqaLITZVMs8CB0oXpX0rroGygwXAqiimIQQIm2U/rP+sUI9zIE7CLvdrIFZqznSYcAN7zfTMBGU7yjDqT9Gf47MmfcQ4BwvixnhcoYIH2Qpy8TIJf+qpR7vsYsYd32MTMuwQxbL5wglRhUfSU8+SAh1xG+eAfOcY3F+8nx0kWGqZwkzss0s9aAP6KXn7n1c/TzBeeB+AwQCdfQ3bMMqgdCzkucu9/Em3D2clNOu8txrQBJ7jqNeXW+0GOYR4ATzhn3hKYB5JZzVny7Ewxo9X54ELWOEWaCzrmQvsBCDP8yl8Qgi+3C4WyPeLC7F0EFnlPtpCIrZTjEKoBpOAR7fxgLoKP12GqeQzNrET2dfxEs4jzZg/9NEbw+X3N7fjyHzpu6IUgF9LcAAAAAElFTkSuQmCC){width=4cm align=center}

![][image]

[image]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAeCAQAAADArVVKAAABJWlDQ1BJQ0MgcHJvZmlsZQAAKJGdkD1KxFAUhb/MiH9opVioRQoRLAZsTGUzKgRBIcYRHK0ySQYHkxiSDIM7cCe6mCkEwR24AQVrz4sWFqbxweV8XO49570HLTsJ03JmF9KsKly/27/sX9lzb7TZYIEdtoOwzLued0Lj+XzFMvrSMV7Nc3+e2SguQ+lUlYV5UYG1L3YmVW5Yxeptzz8UP4jtKM0i8ZN4K0ojw2bXT5Nx+ONpbrMUZxfnpq/axOWYUzxsBowZkVDRkWbqHOGwJ3UpCLinJJQmxOpNNFNxIyrl5HIg6ol0m4a89TrPU8pAHiN5mYQ7UnmaPMz/fq99nNWb1to0D4qgbrVVreEQ3h9huQ8rz7B43ZA1//ttDTNOPfPPN34BFKlQexFMbQcAAAACYktHRAD/h4/MvwAAAAlwSFlzAAALEwAACxMBAJqcGAAAAAd0SU1FB+UDGwkqM5WPHG4AAAAZdEVYdENvbW1lbnQAQ3JlYXRlZCB3aXRoIEdJTVBXgQ4XAAACJUlEQVRIx7WWTUiUQRjHf/PuqxWWepMQiXSRLuYlECIvBuJND5UFtgoGdSkEsQ6CVIcgO5id6lCiYHSoWyhFQpB06VCelDYK6cOi2JJ2i9F9dzo4TO375bbu+8xl5vnPzPPM8zkCiwRD1CMoLSneco0p6COLimhk6RUsso/nXCdb4hfYDHCQJXBQHCEKOorCsbBQrEciYB2wLCImO29lUa1nkoxrZzk79SyDzEMq9S0Oq/7hlKNTz3eTRiKRJKl1KXJXI5I+l1Jzmv/SpW4XCmV59NwYcSbZYbiCQboN5j5TpvllfvoH+6CNKwbt4BJFeiv4mOAsCQAamWBbsU4O0yvGOC1UMU1NqaLITZVMs8CB0oXpX0rroGygwXAqiimIQQIm2U/rP+sUI9zIE7CLvdrIFZqznSYcAN7zfTMBGU7yjDqT9Gf47MmfcQ4BwvixnhcoYIH2Qpy8TIJf+qpR7vsYsYd32MTMuwQxbL5wglRhUfSU8+SAh1xG+eAfOcY3F+8nx0kWGqZwkzss0s9aAP6KXn7n1c/TzBeeB+AwQCdfQ3bMMqgdCzkucu9/Em3D2clNOu8txrQBJ7jqNeXW+0GOYR4ATzhn3hKYB5JZzVny7Ewxo9X54ELWOEWaCzrmQvsBCDP8yl8Qgi+3C4WyPeLC7F0EFnlPtpCIrZTjEKoBpOAR7fxgLoKP12GqeQzNrET2dfxEs4jzZg/9NEbw+X3N7fjyHzpu6IUgF9LcAAAAAElFTkSuQmCC>


### Code

To create `inline code`, wrap with backticks `.

	`inline code`

To create a code block, either indent each line by 4 spaces, or place 3 backticks \`\`\` on a line above and below the
code block.

	```
	code block
	```

## Extended Syntax

Advanced features that build on the basic Markdown syntax.

### Multi-Files

The *Markdown* parser supports organizing the documentation in multiple files. Each file can be included by a simple
link with the relative path. The documentation generator will merge all files into a single *markdown* source. Each file
should be designed as independent file, the generator takes care about building the correct structure of chapters.

	[Title](local_path/to_file.md)

!w An included file will be handled as sub-chapter of the current chapter.

### Comments

Most Markdown implementatios support comments.

	[TEXT]: # (Some comment)

[comment]: # (Still another comment)

### Internal Links

Markdown allows to link to internal headers using as link the header name in lower case, prefixed with a hash, and
replacing all spaces with an sign (-).

	[text](#header-of-document)
	
	### Header of Document

The renderer changes to the local references automatically, that you can directly use the header. Nevertheless you
should avoid headers with special characters.

	[text](#Header of Document)
	
	### Header of Document

### Tables

To add a table, use three or more hyphens (---) to create each column’s header, and use pipes (|) to separate each
column. You can optionally add pipes on either end of the table. We support the extended syntax to align the columns. By
default the columns are left aligned, using colons before or after the hypens it is possible to define a left (:---),
center (:---:) or right alignment (---:)

	| Auto | Center Alignment | Left Alignment | Right Alignment |
	| ---- | :--------------: | :------------- | --------------: |
	| Auto | Center | Left | Right |

| Auto | Center Alignment | Left Alignment | Right Alignment |
| ---- | :--------------: | :------------- | --------------: |
| Auto | Center | Left | Right |


Sizing of of the single table columns is not supported by markdown. We extended the tables and introduced the definition
of the column width, using the number of hyphens (---) in the header separator. The number of hyphens in a single column
relative to the sum of all hyphens in all columns define the relative width to the whole table.

### Alerts

Alerts are an extension to provide customizable blockquotes. 4 different kind of alerts are supported, that are rendered
with a background and highlight color. The alert message have to start with **!** at the beginning of the line with a
letter defining the alert message.

	!n Note

!n Note

	!s Success

!s Success

	!w Warning

!w Warning

	!e Error

!e Error

Sometimes multiline alerts are need. Multiline alert starts with double **!!**. The message will be terminated with a
single line of **!!**.

	!!w Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.
	
	Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.
	!!

!!w Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore
magna aliquyam erat, sed diam voluptua.

Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore
magna aliquyam erat, sed diam voluptua.
!!

### Symbols

Our implementation allows to define placeholders for font symbols. The symbol name must be defined between 2 @.

	@web@

@web@ @mobile@ @config@ @link@

As font the [Material Icons Font][3] is used.


[3]: https://material.io/resources/icons/ "Material Icons Font"

### Formatted Code

The implementation supports the extended syntax with 3 tilde (~), optionally to defining the the code formatter.

	~~~ini
    ; last modified 1 April 2001 by John Doe
    [owner]
    name = John Doe
    organization = Acme Widgets Inc.
    
    [database]
    ; use IP address in case network name resolution is not working
    server = 192.0.2.62     
    port = 143
    file = "payroll.dat"
	~~~

~~~ini
; last modified 1 April 2001 by John Doe
[owner]
name = John Doe
organization = Acme Widgets Inc.

[database]
; use IP address in case network name resolution is not working
server = 192.0.2.62     
port = 143
file = "payroll.dat"
~~~

~~~yaml
# An employee record
name: Martin D'vloper
job: Developer
skill: Elite
employed: True
foods:
  - Apple
  - Orange
  - Strawberry
  - Mango
languages:
  perl: Elite
  python: Elite
  pascal: Lame
education: |
  4 GCSEs
  3 A-Levels
  BSc in the Internet of Things
~~~

~~~json
{
  "squadName": "Super hero squad",
  "homeTown": "Metro City",
  "formed": 2016,
  "secretBase": "Super tower",
  "active": true,
  "members": [
    {
      "name": "Molecule Man",
      "age": 29,
      "secretIdentity": "Dan Jukes",
      "powers": [
        "Radiation resistance",
        "Turning tiny",
        "Radiation blast"
      ]
    },
    {
      "name": "Madame Uppercut",
      "age": 39,
      "secretIdentity": "Jane Wilson",
      "powers": [
        "Million tonne punch",
        "Damage resistance",
        "Superhuman reflexes"
      ]
    },
    {
      "name": "Eternal Flame",
      "age": 1000000,
      "secretIdentity": "Unknown",
      "powers": [
        "Immortality",
        "Heat Immunity",
        "Inferno",
        "Teleportation",
        "Interdimensional travel"
      ]
    }
  ],
  "from": "https://developer.mozilla.org/en-US/docs/Learn/JavaScript/Objects/JSON"
}
~~~

~~~java
import java.awt.Rectangle;

public class ObjectVarsAsParameters {
    public static void main(String[] args) {
        go();
    }

    public static void go() {
        Rectangle r1 = new Rectangle(0, 0, 5, 5);
        System.out.println("In method go. r1 " + r1 + "\n");
        // could have been
        //System.out.prinltn("r1" + r1.toString());
        r1.setSize(10, 15);
        System.out.println("In method go. r1 " + r1 + "\n");
        alterPointee(r1);
        System.out.println("In method go. r1 " + r1 + "\n");

        alterPointer(r1);
        System.out.println("In method go. r1 " + r1 + "\n");
    }

    public static void alterPointee(Rectangle r)
    {
        System.out.println("In method alterPointee. r " + r + "\n");
        r.setSize(20, 30);
        System.out.println("In method alterPointee. r " + r + "\n");
    }

    public static void alterPointer(Rectangle r)
    {
        System.out.println("In method alterPointer. r " + r + "\n");
        r = new Rectangle(5, 10, 30, 35);
        System.out.println("In method alterPointer. r " + r + "\n");
    }
}
~~~

~~~cpp
#include <iostream>
using namespace std;

int main() {

   int num_int;
   double num_double = 9.99;

   // implicit conversion
   // assigning a double value to an int variable
   num_int = num_double;

   cout << "num_int = " << num_int << endl;
   cout << "num_double = " << num_double << endl;

   return 0;
}
~~~

~~~rust
fn main() {
    // We can use this function here, and define it somewhere later
    fizzbuzz_to(100);
}

// Function that returns a boolean value
fn is_divisible_by(lhs: u32, rhs: u32) -> bool {
    // Corner case, early return
    if rhs == 0 {
        return false;
    }

    // This is an expression, the `return` keyword is not necessary here
    lhs % rhs == 0
}

// Functions that "don't" return a value, actually return the unit type `()`
fn fizzbuzz(n: u32) -> () {
    if is_divisible_by(n, 15) {
        println!("fizzbuzz");
    } else if is_divisible_by(n, 3) {
        println!("fizz");
    } else if is_divisible_by(n, 5) {
        println!("buzz");
    } else {
        println!("{}", n);
    }
}

// When a function returns `()`, the return type can be omitted from the
// signature
fn fizzbuzz_to(n: u32) {
    for n in 1..=n {
        fizzbuzz(n);
    }
}
~~~

### Extensions

Uml Diagrams with PlantUML

~~~uml
hide footbox
actor Caller
box "MultiAsyncExecutableTask"
    participant "Task1" as T1
    participant "Task2" as T2
    participant "Task3" as T3
end box
participant REST

Caller -> T1 : execute()
activate T1

T1 -> REST : GET

T1 -> T2 : execute()
deactivate T1
activate T2

T2 -> REST : GET

T2 -> T3 : execute()
deactivate T2
activate T3

T3 -> REST :  GET

Caller <-- T3 : return
deactivate T3

...500 ms...

T1 <-- REST : 200 OK
activate T1

T1 --> T2 : Result.ok()
deactivate T1

T2 <-- REST : 200 OK
activate T2

T2 --> T3 : Result.ok()
deactivate T2

T3 <-- REST : 200 OK
activate T3

Caller <-- T3 : Result.ok()
deactivate T3
~~~

~~~uml
interface DocumentElement
interface ParagraphElement
interface FlowElement

DocumentElement <|.. Header
DocumentElement <|.. Break
DocumentElement <|.. Table
DocumentElement <|.. ParagraphElement
ParagraphElement <|.. Paragraph
ParagraphElement <|.. BlockQuotes
ParagraphElement <|.. List
ParagraphElement <|.. CodeBlock

Document "1" *--> "*" DocumentElement : contains
Header "1" *--> "1" FlowElement : contains
Paragraph "1" *--> "1" FlowElement : contains
CodeBlock "1" *--> "*" FlowElement : contains
~~~

~~~uml
interface FlowElement
FlowElement <|.. Text
FlowElement <|.. Link
FlowElement <|.. Style
FlowElement <|.. Image
FlowElement "1" *--> "*" FlowElement : contains
~~~

~~~uml
interface DocumentElement
interface ParagraphElement
interface FlowElement

DocumentElement <|.. Header
DocumentElement <|.. Break
DocumentElement <|.. Table
DocumentElement <|.. ParagraphElement
ParagraphElement <|.. Paragraph
ParagraphElement <|.. BlockQuotes
ParagraphElement <|.. List
ParagraphElement <|.. CodeBlock

Document "1" *--> "*" DocumentElement : contains
Header "1" *--> "1" FlowElement : contains
Paragraph "1" *--> "1" FlowElement : contains
CodeBlock "1" *--> "*" FlowElement : contains
~~~

~~~uml
interface FlowElement
FlowElement <|.. Text
FlowElement <|.. Link
FlowElement <|.. Style
FlowElement <|.. Image
FlowElement "1" *--> "*" FlowElement : contains
~~~

~~~uml
class Database {}
class ObjectCollection {}
class ObjectReference {}
class PersistentObject {}
class ObjectInfo {}

hide members

Database   -d-> ObjectCollection : supplies
ObjectCollection -d-> PersistentObject         : manages persistence
ObjectReference  -u-> ObjectCollection         : uses
PersistentObject "1" -u-> "*" ObjectCollection : contains
PersistentObject "1" -l-> "*" ObjectReference  : contains
ObjectInfo       -u-> PersistentObject         : describes
~~~

~~~uml
start
repeat
  :Test something;
    if (Something went wrong?) then (no)
      #palegreen:OK;
      break
    endif
    ->NOK;
    :Alert "Error with long text";
repeat while (Something went wrong with long text?) is (yes) not (no)
->//merged step//;
:Alert "Success";
stop
~~~

~~~uml
participant Participant as Foo
actor       Actor       as Foo1
boundary    Boundary    as Foo2
control     Control     as Foo3
entity      Entity      as Foo4
database    Database    as Foo5
collections Collections as Foo6
queue       Queue       as Foo7
Foo -> Foo1 : To actor
Foo -> Foo2 : To boundary
Foo -> Foo3 : To control
Foo -> Foo4 : To entity
Foo -> Foo5 : To database
Foo -> Foo6 : To collections
Foo -> Foo7: To queue
~~~

~~~jsongraph
<style>
.h1 {
  BackGroundColor
  green
  FontColor
  white
  FontStyle
  italic
}
.h2 {
  BackGroundColor
  red
  FontColor
  white
  FontStyle
  bold
}
</style>
#highlight "lastName"
#highlight "address" / "city" <<h1>>
#highlight "phoneNumbers" / "0" / "number" <<h2>>
{
"firstName": "John",
"lastName": "Smith",
"isAlive": true,
"age": 28,
"address": {
"streetAddress": "21 2nd Street",
"city": "New York",
"state": "NY",
"postalCode": "10021-3100"
},
"phoneNumbers": [
{
"type": "home",
"number": "212 555-1234"
},
{
"type": "office",
"number": "646 555-4567"
}
],
"children": [],
"spouse": null
}
~~~

~~~jsongraph
{
  "firstName": "John",
  "lastName": "Smith",
  "isAlive": true,
  "age": 27,
  "address": {
    "streetAddress": "21 2nd Street",
    "city": "New York",
    "state": "NY",
    "postalCode": "10021-3100"
  },
  "phoneNumbers": [
    {
      "type": "home",
      "number": "212 555-1234"
    },
    {
      "type": "office",
      "number": "646 555-4567"
    }
  ],
  "children": [],
  "spouse": null
}
~~~

~~~ebnf
title LISP Grammar
grammars_expression = atomic_symbol | "(", s_expression, ".", s_expression, ")" | list;
list = "(", s_expression, { s_expression }, ")";
atomic_symbol = letter, atom_part;
atom_part = empty | letter, atom_part | number, atom_part;
letter = ? a-z ?;
number = ? 1-9 ?;
empty = " ";
~~~

~~~regex
!option useDescriptiveNames false

\\d?\\D+\\w*\\W{1,2}|\\s.\\S
~~~

~~~ebnf
title Semantic Versioning 2.0
semver = version_core "-", pre_release "+", build;
~~~

~~~dot
digraph G {

  subgraph cluster_0 {
    style=filled;
    color=lightgrey;
    node [style=filled,color=white];
    a0 -> a1 -> a2 -> a3;
    label = "process #1";
  }

  subgraph cluster_1 {
    node [style=filled];
    b0 -> b1 -> b2 -> b3;
    label = "process #2";
    color=blue
  }
  start -> a0;
  start -> b0;
  a1 -> b3;
  b2 -> a3;
  a3 -> a0;
  a3 -> end;
  b3 -> end;

  start [shape=Mdiamond];
  end [shape=Msquare];
}
~~~

~~~railroad
H2_SELECT =
    'SELECT' [ 'TOP' term ] [ 'DISTINCT' | 'ALL' ] selectExpression {',' selectExpression} \
    'FROM' tableExpression {',' tableExpression} [ 'WHERE' expression ] \
    [ 'GROUP BY' expression {',' expression} ] [ 'HAVING' expression ] \
    [ ( 'UNION' [ 'ALL' ] | 'MINUS' | 'EXCEPT' | 'INTERSECT' ) select ] [ 'ORDER BY' order {',' order} ] \
    [ 'LIMIT' expression [ 'OFFSET' expression ] [ 'SAMPLE_SIZE' rowCountInt ] ] \
    [ 'FOR UPDATE' ];
~~~

The *language* attribute allows to define the formatting for a specific language. *ini*, *xml*, *json*, *java* and *cpp*
are supported. Support for *rest* is missing.

## Rendering Configuration

The *markdown.config* parameter allows to define a custom configuration for rendering.

- **template**: A template the defines the rendering configuration. The template allows to define the default page size
with the *width* and *height* parameters, and the default font configuration with the parameter *font-family*.
- **font**: Declares a named font, composed by different TTF (*True Type Font*) files for each metric. Optionally a
codepoints file can be declared to define named symbols of a symbol font file.
- **style**: Allows to define for each text block a different font configuration, line height or coloring. The *style*
can be applied for text styles, emphasise and special blocks. On blocks the styles supports additional attributes, like
*border*, *padding* or *background-color*
- **page**: defines the setting of a single page. On the page you can redefine the page *size*, the *margin* and 
the *padding*. Each page can have a *head*, *tail*, *start* and *end* block with a *size*. The page *margin* defines the
intend from the page borders, inclusive the blocks. The page *padding* defines the intend of the body from
the blocks. On the page you can define optionally a watermark image or a background color.
On the page it's possible to place static elements. On each block we can define flow elements.
- **page-set**: defines a collection of pages for a specific configuration. Usually you need a single set, but
you can define different sets, if you want a special rendering for chapters.

~~~xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE template>
<template width="210mm" height="297mm" font-family="ubuntu">
    <font name="ubuntu">
        <font-metric uri=":fonts/ubuntu/UbuntuSans-Regular.ttf"/>
        <font-metric uri=":fonts/ubuntu/UbuntuSans-Bold.ttf" bold="true"/>
        <font-metric uri=":fonts/ubuntu/UbuntuSans-Italic.ttf" italic="true"/>
        <font-metric uri=":fonts/ubuntu/UbuntuSans-BoldItalic.ttf" bold="true" italic="true"/>
    </font>
    <font name="victor">
        <font-metric uri=":fonts/victor/VictorMono-Medium.ttf"/>
        <font-metric uri=":fonts/victor/VictorMono-Bold.ttf" bold="true"/>
        <font-metric uri=":fonts/victor/VictorMono-MediumItalic.ttf" italic="true"/>
        <font-metric uri=":fonts/victor/VictorMono-BoldItalic.ttf" bold="true" italic="true"/>
    </font>
    <font name="material" codepoints=":fonts/MaterialIcons-Regular.ttf.codepoints">
        <font-metric uri=":fonts/MaterialIcons-Regular.ttf"/>
    </font>

    <style match="subtitle" color="white" font-weight="bold" font-size="26pt" break-after="page" line-height="1.4em"/>
    <style match="h1" color="#130806" font-weight="bold" text-align="left" break-after="page" font-size="42pt"
           line-height="1.1em"/>
    <style match="h[2-6]" font-size="24pt" font-weight="bold" line-height="1.2em" text-align="start"
           keep-with-next="always"
           color="#130806" space-before="1em * 0.8,1em,1em * 1.2" space-after="0.5em * 0.8,0.5em,0.5em * 1.2"/>

    <style match="p" line-height="1.4em" space-before="0.5em" space-after="0.5em" text-align="justify"/>
    <style match="block" line-height="1.4em" margin-left="0.15in" padding-left="0.7em" border-left="3pt solid #ffbe35"/>
    <style match="link" color="#9c8b54"/>

    <style match="info" margin="0" padding="0.2em,1em" background="#bde5f8" keep-with-next="always"
           border=".5px solid #5bc2f3" border-left=".1in solid #5bc2f3" border-radius=".2em"
           space-before="1.6em,2em,2.4em" space-after="1.6em,2em,2.4em"/>
    <style match="success" margin="0" padding="0.2em,1em" background="#dff2bf" keep-with-next="always"
           border=".5px solid #4f8a10" border-left=".1in solid #4f8a10" border-radius=".2em"
           space-before="1.6em,2em,2.4em" space-after="1.6em,2em,2.4em"/>
    <style match="warning" margin="0" padding="0.2em,1em" background="#ffedbd" keep-with-next="always"
           border=".5px solid #ffbe35" border-left=".1in solid #ffbe35" border-radius=".2em"
           space-before="1.6em,2em,2.4em" space-after="1.6em,2em,2.4em"/>
    <style match="error" margin="0" padding="0.2em,1em" background="#f78082" keep-with-next="always"
           border=".5px solid #cc0013" border-left=".1in solid #cc0013" border-radius=".2em"
           space-before="1.6em,2em,2.4em" space-after="1.6em,2em,2.4em"/>
    <style match="code" text-align="start" line-height="1em" keep-with-next="always"
           padding="0.1in" font-family="victor" font-size="9pt"
           space-before="0.8em,1em,1.2em" space-after="0.8em,1em,1.2em"/>
    <style match="styled" padding="0.2em,0.2em"
           font-family="ubuntu" background="#eeeeee"
           border="0.5pt solid #aaaaaa" border-radius="0.2em"/>

    <page name="cover" padding="210mm,0,70mm,2.7in" background=":background.jpg">
        <region position="top" extent="297mm">
            <column top="187mm" left="2.5in" right="0" bottom="70mm" background="#130806">
                <column top="0" left="0.2in" right="0.2in" bottom="0">
                    <row color="#cfa721" font-size="48pt" font-weight="bold" line-height="1.4em">User Manual</row>
                </column>
            </column>
        </region>
    </page>

    <page name="title" padding="15cm,1in,10cm,1in" background="#cfa721">
        <region position="top" extent="107mm">
            <column top="1in" left="1in" color="#130806" font-weight="bold" text-align="left" font-size="256pt"
                    line-height="1.5em">
                <row>
                    {{$CHAPTER}}
                </row>
            </column>
        </region>
        <region position="bottom" extent="107mm" column-count="2" column-gap="12pt">
            <column top=".15in" left="1in" right="1in">
                <br style="solid" size="1pt" color="#000000"/>
            </column>
        </region>
    </page>

    <page name="odd" padding="0.75in">
        <region position="top" extent="8mm">
            <column top="0mm" left="130mm" right="10mm" bottom="0mm" background="#cfa721">
                <row color="#130806" font-size="13pt" text-align="center" top="0.2em">{{$TITLE}}</row>
            </column>
        </region>
        <region position="bottom" extent="10mm">
            <column top="0mm" left="190mm" right="10mm" bottom="0mm" background="#cfa721">
                <row color="#130806" text-align="center" top="0.5em">{{$PAGE_NUMBER}}</row>
            </column>
        </region>
    </page>

    <page name="even" padding="0.75in">
        <region position="top" extent="8mm">
            <column top="0mm" left="10mm" right="130mm" bottom="0mm" background="#cfa721">
                <row color="#130806" font-size="13pt" text-align="center" top="0.2em">{{$TITLE}}</row>
            </column>
        </region>
        <region position="bottom" extent="10mm">
            <column top="0mm" left="10mm" right="190mm" bottom="0mm" background="#cfa721">
                <row color="#130806" text-align="center" top="0.5em">{{$PAGE_NUMBER}}</row>
            </column>
        </region>
    </page>

    <page name="blank"/>

    <page-set name="book">
        <page-entry name="cover" match="First"/>
        <page-entry name="odd" match="Odd"/>
        <page-entry name="even" match="Even"/>
        <page-entry name="blank" match="Blank"/>
    </page-set>

    <page-set name="chapter">
        <page-entry name="title" match="First"/>
        <page-entry name="odd" match="Odd"/>
        <page-entry name="even" match="Even"/>
        <page-entry name="blank" match="Blank"/>
    </page-set>

    <page-set name="standard">
        <page-entry name="odd" match="Odd"/>
        <page-entry name="even" match="Even"/>
        <page-entry name="blank" match="Blank"/>
    </page-set>
</template>
~~~

Currently the renderer support 3 different fonts:

- TEXT: Defines the font for the default text.
- MONO: Defines the font for the code snippets.
- SYMBOLS: Defines the symbol font, used for font icons.
