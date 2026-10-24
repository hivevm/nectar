# Diagrams

Diagrams are rendered by the engines configured in `nectar.toml`.

```dot
digraph { rankdir=LR; Markdown -> Document -> HTML }
```

```uml
Author -> Nectar : edit
Nectar -> Browser : reload
```

```ebnf
document = { chapter } ;
chapter = heading, { block } ;
```

Railroad diagrams are drawn in-process; `\` continues a rule on a new line.

```railroad
list = item { ',' item } ;
item = name [ '=' value ] \
       [ '(' ( option | (? any text ?) ) ')' ] ;
```

A grammar file embedded through an image link:

![Lists](../code/list.railroad)
