// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document.fo;

import org.hivevm.doc.fo.Fo;
import org.hivevm.doc.fo.writer.*;
import org.hivevm.document.TextSpan;

import java.util.HashSet;
import java.util.Stack;

/**
 * The {@code FoRendererFlow} class implements the {@link Flow.Visitor} interface for rendering
 * flows into Apache Formatting Object (FO) nodes. This is used to transform structured flow objects
 * (text, images, links, styles, etc.) into FO-compatible output for document generation.
 */
class FlowRenderer {

    private final FoRenderer renderer;
    private final Stack<FoNode> nodes;

    /**
     * Constructs an instance of the FoRendererFlow class.
     */
    public FlowRenderer(FoNode node, FoRenderer renderer) {
        this.renderer = renderer;
        this.nodes = new Stack<>();
        this.nodes.push(node);
    }

    /**
     * Processes a {@link TextSpan} node and appends its content to an {@link FoNode}. Renders the
     * text content using template symbols and delegates handling of child flows.
     */
    public final void render(TextSpan text) {
        var string = text.text() != null ? text.text() : "";//PageUtil.encode(text.text());
        if (text.isNewLine())
            FoBlock.block(nodes.peek());
        else if (text.isEnd())
            nodes.pop();
        else if (text.isLink()) {
            var destination = text.getLinkUrl();
//            if (destination.startsWith("#"))
//                destination = document.getId(destination);

            var parent = nodes.peek();
            var basicLink = new FoBasicLink(parent);
            basicLink.setDestination(destination);
            renderer.renderStyle(basicLink);
            nodes.push(basicLink);

            if (text.getLinkTitle() != null) {
                var id = String.format("(%s)", text.getLinkIndex());
                var footnote = new FoFootnote(id, parent);
                var content = footnote.getBody().addContent(" ");

                var fn = new FoBasicLink(content);
                renderer.renderStyle(fn);
                fn.setDestination(destination);
                fn.addText(text.getLinkTitle());
            }
        } else if (text.isImage()) {
            var data = nodes.peek();
            if (text.getImageAlign() != null) {
                var align = text.getImageAlign();
                switch (text.getImageAlign()) {
                    case "float-left":
                        var floating = new FoNode("float", data);
                        floating.set("float", "left");
                        data = FoBlock.block(floating);
                        data.set("padding", "5px");
                        break;
                    case "float-right":
                        floating = new FoNode("float", data);
                        floating.set("float", "right");
                        data = FoBlock.block(floating);
                        data.set("padding", "5px");
                        break;
                    default:
                        data.set("text-align", text.getImageAlign());
                }
            }

            var graphic = new FoExternalGraphic(data);
            graphic.setURL(text.getImageUri());
            graphic.setSize(text.getImageWidth(), text.getImageHeight());
            graphic.set("content-width", "scale-to-fit");
            graphic.set("content-height", "scale-to-fit");
            graphic.set("max-width", "95%");

            if (text.getImageTitle() != null) {
                String title = Fo.encode(text.getImageTitle());
                FoBlock block = FoBlock.block(data);
                block.setFontStyle("italic").addContent(title);
            }
        } else if (!string.isEmpty()) {
            renderer.template().getSymbols()
                    .forEach(string, (s, f) -> {
                        if (f == null)
                            nodes.peek().addText(s);
                        else
                            nodes.peek().create("character").set("font-family", f).set("character", s);
                    });
        } else {
            var inline = FoBlock.inline(nodes.peek());
            nodes.push(inline);

            if (text.isBold())
                inline.setFontWeight("bold");
            if (text.isItalic())
                inline.setFontStyle("italic");
            if (text.style() != null)
                inline.setColor(text.style().color());

            var decorations = new HashSet<String>();
            if (text.isOverline())
                decorations.add("overline");
            if (text.isUnderline())
                decorations.add("underline");
            if (text.isStrikethrough())
                decorations.add("line-through");
            if (!decorations.isEmpty())
                inline.setTextDecoration(String.join(" ", decorations));
            if (text.isCode())
                renderer.renderStyle(inline, "styled");
        }
    }
}
