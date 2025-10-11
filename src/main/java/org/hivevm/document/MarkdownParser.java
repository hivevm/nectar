// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

import org.commonmark.ext.task.list.items.TaskListItemMarker;
import org.commonmark.node.*;
import org.commonmark.node.Heading;
import org.commonmark.node.Image;
import org.commonmark.node.Paragraph;
import org.hivevm.commonmark.Markdown;
import org.hivevm.commonmark.MarkdownVisitor;
import org.hivevm.commonmark.alert.AlertBlock;
import org.hivevm.commonmark.image.ImageAttributes;
import org.hivevm.commonmark.marker.Marker;
import org.hivevm.commonmark.table.Table;

import java.io.IOException;
import java.io.InputStream;
import java.util.Stack;

public class MarkdownParser extends MarkdownVisitor {

    private final Document.Builder builder;

    private final Stack<Object> stack = new Stack<>();

    public MarkdownParser() {
        this(new Document.Builder());
    }

    private MarkdownParser(Document.Builder builder) {
        this.builder = builder;
        this.builder.setTitle("TITLE");
        this.builder.setSubTitle("SUBTITLE");
    }

    @Override
    protected void visitOrTraverse(Node node, String name) {
        System.out.println(name);
        super.visitOrTraverse(node, name);
    }

    /**
     * Visit a {@link org.commonmark.node.Document} node.
     */
    @Override
    public final void visit(org.commonmark.node.Document node) {
        stack.push(builder);
        visitChildren(node);
        stack.pop();
    }

    @Override
    public void visit(Paragraph node) {
        var builder = (ContainerElement.Builder) stack.peek();
        var paragraph = builder.addParagraph();
        stack.push(paragraph);
        visitChildren(node);
        stack.pop();
        paragraph.build();
    }

    @Override
    public void visit(Heading node) {
        var heading = builder.addHeader(node.getLevel());
        stack.push(heading);
        visitChildren(node);
        stack.pop();
        heading.build();
    }

    @Override
    public void visit(Text node) {
        var parent = (TextBuilder<?>) stack.peek();
        var builder = new TextSpan.Builder(node.getLiteral());
        stack.push(builder);
        visitChildren(node);
        stack.pop();
        parent.addText(builder.build());
    }

    @Override
    public void visit(Emphasis node) {
        var parent = (TextBuilder<?>) stack.peek();
        var builder = new TextSpan.Builder();
        parent.addText(builder.italic().build());

        visitChildren(node);

        builder = new TextSpan.Builder();
        parent.addText(builder.end().build());
    }

    @Override
    public void visit(StrongEmphasis node) {
        var parent = (TextBuilder<?>) stack.peek();
        var builder = new TextSpan.Builder();
        parent.addText(builder.bold().build());

        visitChildren(node);

        builder = new TextSpan.Builder();
        parent.addText(builder.end().build());
    }

    @Override
    public void visit(Link node) {
        var parent = (TextBuilder<?>) stack.peek();
        var builder = new TextSpan.Builder();
        builder.link(node.getDestination(), node.getTitle(), this.builder);
        var link = builder.build();
        parent.addText(link);

        visitChildren(node);

        builder = new TextSpan.Builder();
        parent.addText(builder.end().build());
    }

    @Override
    public void visit(LinkReferenceDefinition node) {
    }

    @Override
    public void visit(Image node) {
        var parent = (TextBuilder<?>) stack.peek();
        String title = null;
        String align = null;
        String width = null;
        String height = null;

        if (node.getFirstChild() instanceof Text text)
            title = text.getLiteral();
        if (node.getLastChild() instanceof ImageAttributes attrs) {
            align = attrs.getAttributes().get("align");
            width = attrs.getAttributes().get("width");
            height = attrs.getAttributes().get("height");
        }

        var builder = new TextSpan.Builder();
        builder.image(node.getDestination(), title, align, width, height);
        parent.addText(builder.build());
    }

    @Override
    public void visit(Code node) {
        var parent = (TextBuilder<?>) stack.peek();
        var builder = new TextSpan.Builder();
        parent.addText(builder.code().build());

        builder = new TextSpan.Builder(node.getLiteral());
        parent.addText(builder.build());

        builder = new TextSpan.Builder();
        parent.addText(builder.end().build());
    }

    @Override
    public void visit(SoftLineBreak node) {
        var parent = (TextBuilder<?>) stack.peek();
        var builder = new TextSpan.Builder(" ");
        parent.addText(builder.build());
    }

    @Override
    public void visit(HardLineBreak node) {
        var parent = (TextBuilder<?>) stack.peek();
        var builder = new TextSpan.Builder();
        parent.addText(builder.newLine().build());
    }

    @Override
    public void visit(ThematicBreak node) {
        builder.addPageBreak().build();
    }

    @Override
    public void visit(IndentedCodeBlock node) {
        var builder = (ContainerElement.Builder) stack.peek();
        builder.addCodeBlock(node.getLiteral(), null).build();
    }

    @Override
    public void visit(FencedCodeBlock node) {
        var builder = (ContainerElement.Builder) stack.peek();
        builder.addCodeBlock(node.getLiteral(), node.getInfo()).build();
    }

    @Override
    public void visit(BlockQuote node) {
        var builder = (ContainerElement.Builder) stack.peek();
        var paragraph = builder.addBlock(null);
        stack.push(paragraph);

        visitChildren(node);

        stack.pop();
        paragraph.build();
    }

    @Override
    public void visit(BulletList node) {
        var builder = (ContainerElement.Builder) stack.peek();
        var paragraph = builder.addList(ListElement.ListType.UNORDERED);
        stack.push(paragraph);

        visitChildren(node);

        stack.pop();
        paragraph.build();
    }

    @Override
    public void visit(OrderedList node) {
        var builder = (ContainerElement.Builder) stack.peek();
        var paragraph = builder.addList(ListElement.ListType.ORDERED);
        stack.push(paragraph);

        visitChildren(node);

        stack.pop();
        paragraph.build();
    }

    @Override
    public void visit(ListItem node) {
        var builder = (ListElement.Builder) stack.peek();
        var paragraph = builder.addItem();
        stack.push(paragraph);

        visitChildren(node);

        stack.pop();
        paragraph.build();
    }

    /**
     * Visit a {@link HtmlInline} node.
     */
    @Override
    public final void visit(HtmlInline node) {
        var parent = (TextBuilder<?>) stack.peek();
        var builder = new TextSpan.Builder(node.getLiteral());
        parent.addText(builder.build());
    }

    /**
     * Visit a {@link HtmlBlock} node.
     */
    @Override
    public final void visit(HtmlBlock node) {
        super.visit(node);
    }

    /**
     * Visit a {@link CustomNode} node handles additional emphasis nodes.
     */
    @Override
    public final void visit(CustomNode node) {
        if (node instanceof Marker marker) {
            var parent = stack.peek();
            if (parent instanceof TextBuilder<?> head) {
                var builder = new TextSpan.Builder();
                switch (marker.getDecoration()) {
                    case Overline -> builder.overline();
                    case Underline -> builder.underline();
                    case Highlight -> builder.overline().underline();
                    case Strikethrough -> builder.strikethrough();
                }
                head.addText(builder.build());

                visitChildren(node);

                builder = new TextSpan.Builder();
                head.addText(builder.end().build());
            }
        } else if (node instanceof TaskListItemMarker task) {
            var parent = (ListElement.ListItem.Builder) stack.peek();
            parent.setMarker(task.isChecked() ? ListElement.ListMarker.CHECKED : ListElement.ListMarker.UNCHECKED);
        } else if (node instanceof Table.Row) {
            var parent = (org.hivevm.document.Table.Builder) stack.peek();
            parent.addRow(org.hivevm.document.Table.RowType.BODY);
            visitChildren(node);
        } else if (node instanceof Table.Cell cell) {
            var parent = (org.hivevm.document.Table.Builder) stack.peek();
            var builder = parent.addCell();
            stack.push(builder);
            visitChildren(node);
            stack.pop();
            builder.build();
        } else {
            super.visit(node);
        }
    }

    @Override
    public void visit(CustomBlock node) {
        if (node instanceof Table table) {
            var parent = (Document.Builder) stack.peek();
            var builder = parent.addTable();
            for (var i = 0; i < table.columnCount(); i++)
                builder.addColumn(i, table.getWidth(i), table.getAlign(i).name());
            builder.addRow(org.hivevm.document.Table.RowType.HEAD);
            for (var i = 0; i < table.columnCount(); i++) {
                var cell = builder.addCell();
                cell.addText(new TextSpan.Builder(table.getTitle(i)).build());
                cell.build();
            }
            stack.push(builder);

            visitChildren(node);

            stack.pop();
            builder.build();
        } else if (node instanceof AlertBlock alert) {
            var style = switch (alert.getType()) {
                case SUCCESS -> "SUCCESS";
                case WARNING -> "WARNING";
                case ERROR -> "ERROR";
                case NOTE -> "INFO";
            };

            var builder = (ContainerElement.Builder) stack.peek();
            var paragraph = builder.addBlock(style);
            stack.push(paragraph);
            visitChildren(node);
            stack.pop();
            paragraph.build();
        } else {
            super.visit(node);
        }
    }

    /**
     * Parses a Document from the text.
     */
    public static Document parse(String text) throws IOException {
        var builder = new Document.Builder();
        var node = Markdown.parse(text);
        node.accept(new MarkdownParser(builder));
        return builder.build();
    }

    /**
     * Parses a Document from the text.
     */
    public static Document parse(InputStream stream) throws IOException {
        var text = new String(stream.readAllBytes());
        return MarkdownParser.parse(text);
    }
}
