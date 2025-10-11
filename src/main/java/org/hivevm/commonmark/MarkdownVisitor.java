// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.commonmark;

import java.util.HashMap;
import java.util.regex.Pattern;
import org.commonmark.node.AbstractVisitor;
import org.commonmark.node.BlockQuote;
import org.commonmark.node.BulletList;
import org.commonmark.node.Code;
import org.commonmark.node.CustomBlock;
import org.commonmark.node.CustomNode;
import org.commonmark.node.Document;
import org.commonmark.node.Emphasis;
import org.commonmark.node.FencedCodeBlock;
import org.commonmark.node.HardLineBreak;
import org.commonmark.node.Heading;
import org.commonmark.node.HtmlBlock;
import org.commonmark.node.HtmlInline;
import org.commonmark.node.Image;
import org.commonmark.node.IndentedCodeBlock;
import org.commonmark.node.Link;
import org.commonmark.node.LinkReferenceDefinition;
import org.commonmark.node.ListItem;
import org.commonmark.node.Node;
import org.commonmark.node.OrderedList;
import org.commonmark.node.Paragraph;
import org.commonmark.node.SoftLineBreak;
import org.commonmark.node.StrongEmphasis;
import org.commonmark.node.Text;
import org.commonmark.node.ThematicBreak;
import org.hivevm.commonmark.image.ImageAttributes;

/**
 * A concrete implementation of {@link AbstractVisitor} designed to traverse and process a tree of
 * Markdown nodes. Each node type is visited through specific methods that can be extended for
 * customized behavior. Nodes are traversed in depth-first order.
 */
public class MarkdownVisitor extends AbstractVisitor {

    protected void visitOrTraverse(Node node, String name) {
        visitChildren(node);
    }

    @Override
    public void visit(Document node) {
        visitOrTraverse(node, "Document");
    }

    @Override
    public void visit(Heading node) {
        visitOrTraverse(node, "Heading");
    }

    @Override
    public void visit(Paragraph node) {
        visitOrTraverse(node, "Paragraph");
    }

    @Override
    public void visit(SoftLineBreak node) {
        visitOrTraverse(node, "SoftLineBreak");
    }

    @Override
    public void visit(HardLineBreak node) {
        visitOrTraverse(node, "HardLineBreak");
    }

    @Override
    public void visit(ThematicBreak node) {
        visitOrTraverse(node, "ThematicBreak");
    }

    @Override
    public void visit(BlockQuote node) {
        visitOrTraverse(node, "BlockQuote");
    }

    @Override
    public void visit(Text node) {
        visitOrTraverse(node, "Text");
    }

    @Override
    public void visit(Emphasis node) {
        visitOrTraverse(node, "Emphasis");
    }

    @Override
    public void visit(StrongEmphasis node) {
        visitOrTraverse(node, "StrongEmphasis");
    }

    @Override
    public void visit(Link node) {
        visitOrTraverse(node, "Link");
    }

    @Override
    public void visit(LinkReferenceDefinition node) {
        visitOrTraverse(node, "LinkReferenceDefinition");
    }

    @Override
    public void visit(Image node) {
        visitOrTraverse(node, "Image");
    }

    @Override
    public void visit(BulletList node) {
        visitOrTraverse(node, "BulletList");
    }

    @Override
    public void visit(OrderedList node) {
        visitOrTraverse(node, "OrderedList");
    }

    @Override
    public void visit(ListItem node) {
        visitOrTraverse(node, "ListItem");
    }

    @Override
    public void visit(Code node) {
        visitOrTraverse(node, "Code");
    }

    @Override
    public void visit(IndentedCodeBlock node) {
        visitOrTraverse(node, "IndentedCodeBlock");
    }

    @Override
    public void visit(FencedCodeBlock node) {
        visitOrTraverse(node, "FencedCodeBlock");
    }

    /**
     * Visit the provided {@link HtmlBlock} node. This method parses the HTML block content to
     * extract and process specific attributes and elements, including handling `<img>` tags.
     */
    @Override
    public void visit(HtmlBlock node) {
        var literal = node.getLiteral().trim();
        if (literal.startsWith("<img") && literal.endsWith(">")) {
            var matcher = Pattern.compile("(\\w+)=\"([^\"]+)\"").matcher(literal);
            var image = new Image();
            var text = new Text();
            var attrs = new HashMap<String, String>();
            while (matcher.find()) {
                switch (matcher.group(1)) {
                    case "src":
                        image.setDestination(matcher.group(2));
                        break;
                    case "width":
                        attrs.put("width", matcher.group(2));
                        break;
                    case "align":
                        attrs.put("align", matcher.group(2));
                        break;
                    case "alt":
                        text.setLiteral(matcher.group(2));
                        break;
                    default:
                }
            }
            if (text.getLiteral() != null)
                image.appendChild(text);
            if (!attrs.isEmpty())
                image.appendChild(new ImageAttributes(attrs));
            var p = new Paragraph();
            p.appendChild(image);
            visit(p);
        }
        else
            visitOrTraverse(node, "HtmlBlock");
    }

    @Override
    public void visit(HtmlInline node) {
        visitOrTraverse(node, "HtmlInline");
    }

    @Override
    public void visit(CustomBlock node) {
        visitOrTraverse(node, "CustomBlock");
    }

    @Override
    public void visit(CustomNode node) {
        visitOrTraverse(node, "CustomNode");
    }

    @Override
    public final void visitChildren(Node node) {
        super.visitChildren(node);
    }
}
