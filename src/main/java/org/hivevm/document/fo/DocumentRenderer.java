// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document.fo;

import org.hivevm.doc.fo.Fo;
import org.hivevm.doc.fo.writer.*;
import org.hivevm.document.*;

import java.util.Properties;

/**
 * The FoDocument class represents a visitor that processes a Document and its nodes (content
 * elements) to generate an XSL-FO (Extensible Stylesheet Language Formatting Objects) structure.
 * This structure can then be used for generating printable documents such as PDFs.
 * <p>
 * The class supports rendering various structural elements such as headers, paragraphs, tables,
 * lists, and other content types. It also manages the creation of specialized sections like the
 * Table of Contents and applies styles to ensure proper formatting of the generated document.
 */
public class DocumentRenderer implements DocumentVisitor<FoRenderer> {

    private TOCRenderer toc;

    private DocumentRenderer() {

    }

    /**
     * Visits a {@link Document} instance and processes its content to generate the FO structure.
     * This method handles the creation of the Table of Contents, rendering the cover page, and
     * processing the document elements.
     */
    public static void render(Document document, FoRenderer writer) {
        // Renders the cover page
        var properties = new Properties();
        properties.put("TITLE", document.title());
        properties.put("PAGE_NUMBER", "<fo:page-number/>");

        var renderer = new DocumentRenderer();
        renderer.toc = TOCRenderer.create(document);
        renderer.toc.renderBookmark(document, writer);

        writer.createFlow(Fo.BOOK_ID, Fo.PAGESET_BOOK, properties);
        var block = FoBlock.block(writer.flow());

        writer.renderStyle(block, "subtitle");
        document.elements().forEach(c -> c.accept(renderer, writer));

        renderer.toc.renderToc(document, writer);
    }

    @Override
    public void visit(Heading heading, FoRenderer writer) {
        Properties properties = new Properties();
        properties.put("CHAPTER", heading.level() /* PageUtil.getNumber(node)*/);
        properties.put("TITLE", toc.getTitle(heading));

        if (heading.level() == 1) {
            writer.createFlow(heading.id(), Fo.PAGESET_CHAPTER, properties);
        }

        FoBlock content = FoBlock.block(writer.flow());
        content.setId(heading.id());
        if (heading.level() == 2) {
            content.setBreakBefore("page");
        }

//        // Partial Table of Content
//        FoBlock index = flow.addBlock()
//                .setColor("#130806")
//                .setFontSize("12pt")
//                .setTextAlign("left")
//                .setLineHeight("1.8em");
//
//        doc.stream()
//                .filter(n -> n instanceof Heading)
//                .map(n -> (Heading) n)
//                .forEach(p -> FoDocumentRenderer.createTocEntry(p.getId(), PageUtil.encode(p.getTitle()), index.addBlock()));

        writer.renderStyle(content, "h" + heading.level());
        content.addText(toc.getChapter(heading));
        var flow = new FlowRenderer(content, writer);
        heading.textSpans().forEach(flow::render);
    }

    @Override
    public void visit(Paragraph paragraph, FoRenderer writer) {
        FoBlock content = FoBlock.block(writer.top());
        writer.renderStyle(content, "p");
        var flow = new FlowRenderer(content, writer);
        paragraph.textSpans().forEach(flow::render);
    }

    @Override
    public void visit(PageBreak pageBreak, FoRenderer writer) {
        FoBlock.block(writer.top()).setPageBreak();
    }

    @Override
    public void visit(Image image, FoRenderer writer) {
        FoBlock content = FoBlock.block(writer.top());

        var graphic = new FoExternalGraphic(content);
        graphic.setURL(image.getSrc());
        graphic.setSize(
                image.getWidth() > 0 ? "" + image.getWidth() : null,
                image.getHeight() > 0 ? "" + image.getHeight() : null);
        graphic.set("content-width", "scale-to-fit");
        graphic.set("content-height", "scale-to-fit");
        graphic.set("max-width", "95%");
    }

    @Override
    public void visit(Block block, FoRenderer writer) {
        var content = FoBlock.block(writer.top());
        if (block.getCssClass() == null) // Blockquote
            writer.renderStyle(content, "block");
        else {
            writer.renderStyle(content, block.getCssClass().toLowerCase());
            content.setKeepWithPage("always");
        }

        writer.push(content);
        block.getChildren().forEach(c -> c.accept(this, writer));
        writer.pop();
    }

    @Override
    public void visit(CodeBlock codeBlock, FoRenderer writer) {
        FoBlock content = FoBlock.block(writer.top());

        writer.renderStyle(content, "code");

        content.setWarp("wrap");
        content.setLineFeed("preserve");
// TODO: Disabled due XML
//        content.setKeepWithPage(lines.size() > 10 ? "auto" : "always");
        content.setWhiteSpaceCollapse("false");
        content.setWhiteSpaceTreatment("preserve");
        content.setBorderRadius("0.2em");

        for (var line : codeBlock.getLines()) {
            var row = FoBlock.block(content);
            var flow = new FlowRenderer(row, writer);
            if (line.span().isEmpty())
                row.addText(" ");
            else
                line.span().forEach(flow::render);
        }
    }

    @Override
    public void visit(ListElement list, FoRenderer writer) {
        FoListBlock block = new FoListBlock(writer.top());

        block.setEndIndent("1em")
                .setDistanceBetweenStarts("1.0em")
                .setLabelSeparation("0.2em");

        int index = 0;
        for (var child : list.getItems()) {
            String label = list.isOrdered() ? String.format("%s.", ++index) : "•";
            FoListItem item = new FoListItem(label, block);

            // TODO: set checked

            writer.push(item.getContent());
            child.getChildren().forEach(i -> i.accept(this, writer));
            writer.pop();
        }

    }

    /**
     * Renders an alert {@link Table} node.
     */
    @Override
    public final void visit(Table table, FoRenderer writer) {
        FoTable foTable = new FoTable(writer.top());
        foTable.setTableLayout("fixed");

//        if (table.isVirtual()) {
//            if (table.getBorderColor() != null) {
//                foTable.setBorder(".5px", "solid", table.getBorderColor());
//            }
//            foTable.setBackgroundColor(table.getBackgroundColor());
//        } else {
        foTable.setSpace("1em");
        foTable.setBorderTop("1px", "solid", "#777777");
        foTable.setBorderBottom("1px", "solid", "#777777");
        foTable.setBorderBefore("retain").setBorderCollapse("collapse");
//        }

        // Render columns
        for (var column : table.getColumns()) {
            foTable.addColumn("" + (column.index() + 1), table.getColumnWidth(column.index()) + "%");
        }

        FoTableArea area = null;
        Table.RowType type = null;

        // Render the areas
        int i = 0;
        for (var r : table.getRows()) {
            if (area == null || type != r.rowType()) {
                type = r.rowType();
                area = switch (type) {
                    case HEAD -> foTable.addHead();
                    case TAIL -> foTable.addFoot();
                    default -> foTable.addBody();
                };
            }

            FoTableRow row = area.addRow();
            switch (type) {
                case HEAD:
                    row.setFontWeight("bold");
                    row.setBackgroundColor("#cccccc");
                    break;

                case BODY:
//                        if (!table.isVirtual())
                    row.setBackgroundColor(((i++ % 2) == 1) ? "#eeeeee" : "#f7f7f7");
                    break;

                default:
                    break;
            }

            // Render Cells
            for (int j = 0; j < r.getCells().size(); j++) {
                var c = r.getCells().get(j);
                var a = table.getColumns().get(j).align().toLowerCase();
                FoTableCell cell = row.addCell();
                cell.setRowSpan(c.getRowspan());
                cell.setColSpan(c.getColspan());
                cell.setTextAlign("auto".equals(a) ? "" : a);

//                    if (table.isVirtual()) {
//                        cell.setBorder("0", "none", "transparent");
//                    } else {
                cell.setDisplayAlign("center");
                cell.setPadding("2pt");
                cell.setBorderBottom("0.5px", "solid", "#777777");
//                    }

                var block = FoBlock.block(cell);
                var flow = new FlowRenderer(block, writer);
                c.getContent().forEach(flow::render);
            }
        }
    }
}
