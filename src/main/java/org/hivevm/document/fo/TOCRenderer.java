// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document.fo;

import org.hivevm.doc.fo.Fo;
import org.hivevm.doc.fo.writer.*;
import org.hivevm.document.Document;
import org.hivevm.document.Heading;
import org.jspecify.annotations.NonNull;

import java.util.*;

/**
 * Represents a Table of Contents (ToC) for a document. This class manages hierarchical
 * relationships between headers in a document and creates bookmarks for easy navigation.
 */
class TOCRenderer {

    private static final int[] ARAB = {1000, 990, 900, 500, 490, 400, 100, 90, 50, 40, 10, 9, 5,
            4, 1};
    private static final String[] ROMAN = {"M", "XM", "CM", "D", "XD", "CD", "C", "XC", "L", "XL",
            "X", "IX", "V", "IV", "I"};

    private final String tocId;
    private final Map<Heading, String> headers;

    public TOCRenderer() {
        this.headers = new HashMap<>();
        this.tocId = Long.toHexString(new Random().nextLong());
    }

    private void set(Heading header, String content) {
        headers.put(header, content.isEmpty() ? "" : content + " ");
    }

    /**
     * Retrieves the chapter content associated with the specified header.
     */
    public final String getChapter(Heading header) {
        return headers.get(header);
    }

    /**
     * Retrieves the encoded and trimmed title of the specified header.
     */
    public final String getTitle(Heading header) {
        return Fo.encode(header.title().trim());
    }

    /**
     * Retrieves the complete chapter title by combining the chapter content and the encoded and
     * trimmed title associated with the specified header.
     */
    public final String getChapterTitle(Heading header) {
        return getChapter(header) + getTitle(header);
    }

    /**
     * Renders the bookmarks for the given document and writes them to the output writer.
     */
    public final void renderBookmark(@NonNull Document doc, FoRenderer writer) {
        try (var tree = new FoNode("bookmark-tree", writer.root())) {
            var bookmarks = new Stack<FoBookmark>();
            bookmarks.push(new FoBookmark(Fo.BOOK_ID, tree).setTitle(doc.title()));
            doc.elements().stream()
                    .filter(n -> n instanceof Heading)
                    .map(n -> (Heading) n)
                    .filter(h -> h.level() > 1)
                    .forEach(h -> renderBookmark(h, tree, bookmarks));
            new FoBookmark(tocId, tree).setTitle("I. - Table of content");
        }
    }

    /**
     * Renders a bookmark structure for a given header node by managing a stack of bookmarks.
     */
    private void renderBookmark(@NonNull Heading node, @NonNull FoNode tree,
                                Stack<FoBookmark> bookmarks) {
        while (bookmarks.size() >= node.level()) {
            bookmarks.pop();
        }

        var bookmark = bookmarks.isEmpty()
                ? new FoBookmark(node.id(), tree)
                : bookmarks.peek().addBookmark(node.id());
        bookmark.setTitle(getChapterTitle(node));
        bookmarks.push(bookmark);
    }

    /**
     * Renders the table of contents (TOC) for the provided document and writes the output using the
     * specified writer. The method processes the headers in the document, organizes them into a
     * hierarchical TOC structure, and applies formatting for display.
     */
    public final void renderToc(Document doc, FoRenderer writer) {
        var title = "Table of Contents";
        var properties = new Properties();
        properties.put("TITLE", title);

        writer.createFlow(tocId, Fo.PAGESET_STANDARD, properties);
        var content = FoBlock.block(writer.flow());
        content.setBreakBefore("page")
                .setSpaceBefore("0.5em", "1.0em", "2.0em")
                .setSpaceAfter("0.5em", "1.0em", "2.0em")
                .setColor("#000000")
                .setTextAlign("left");
        content.setId(tocId);

        var block = content.addBlock();
        block.setSpaceBefore("1.0em", "1.5em", "2.0em");
        block.setSpaceAfter("0.5em").setStartIndent("0pt");
        block.setFontWeight("bold").setFontSize("18pt");
        block.addText(title);

        doc.elements().stream()
                .filter(n -> n instanceof Heading)
                .map(n -> (Heading) n)
                .filter(h -> h.level() > 1)
                .forEach(header -> {
                    int intent = header.level() - 1;

                    var block0 = content.addBlock();
                    block0.setMarginLeft(String.format("%sem", intent));
                    block0.setTextAlignLast("justify");

                    var link = new FoBasicLink(block0);
                    link.setDestination(header.id());

                    var inline = FoBlock.inline(link);
                    inline.setKeepWithNext("always")
                            .addText(Fo.encode(getChapterTitle(header)));

                    var leader = new FoLeader(inline);
                    leader.setPattern("dots")
                            .setWidth("3pt")
                            .setAlign("reference-area")
                            .setPaddingLeftRight("3pt");
                    new FoPageNumberCitation(header.id(), inline);
                });
    }

    private static String asRomanNumber(int number) {
        var result = new StringBuilder();
        int i = 0;
        while ((number > 0) || (TOCRenderer.ARAB.length == (i - 1))) {
            while ((number - TOCRenderer.ARAB[i]) >= 0) {
                number -= TOCRenderer.ARAB[i];
                result.append(TOCRenderer.ROMAN[i]);
            }
            i++;
        }
        return result.toString();
    }

    /**
     * Creates an instance of {@code FoTableOfContent} by analyzing the given document. This method
     * processes the document's structural elements, particularly headers, and organizes them into a
     * hierarchical table of contents structure.
     */
    public static TOCRenderer create(Document doc) {
        var renderer = new TOCRenderer();

        var stack = new Stack<Integer>();
        for (var header : doc.elements().stream()
                .filter(n -> n instanceof Heading)
                .map(n -> (Heading) n).toList()) {

            while (stack.size() > header.level()) {
                stack.pop();
            }

            if (stack.size() < header.level()) {
                while (stack.size() < header.level() - 1) {
                    stack.push(0);
                }
                stack.push(1);
            } else {
                int offset = stack.pop();
                stack.push(offset + 1);
            }

            if (header.level() == 1)
                renderer.set(header, "");
            else {
                var value = "";
                var isRoman = false;
                for (var i = 1; i < stack.size(); i++) {
                    int level = stack.get(i);
                    if (level == 0) {
                        value = "";
                        isRoman = true;
                    } else
                        value += (isRoman ? TOCRenderer.asRomanNumber(level) : level) + ".";
                }
                renderer.set(header, value);
            }
        }

        return renderer;
    }
}
