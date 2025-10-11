// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.commonmark.table;

import org.commonmark.node.Node;
import org.commonmark.parser.InlineParser;
import org.commonmark.parser.SourceLine;
import org.commonmark.parser.SourceLines;
import org.commonmark.parser.block.*;

import java.util.ArrayList;
import java.util.List;

class TableParser extends AbstractBlockParser {

    private final Table table;
    private final List<CharSequence> bodyLines = new ArrayList<>();

    private boolean nextIsSeparatorLine = true;

    private TableParser(List<Integer> width, List<Table.Alignment> align, List<String> title) {
        this.table = new Table(width, align, title);
    }

    @Override
    public boolean canHaveLazyContinuationLines() {
        return true;
    }

    @Override
    public Table getBlock() {
        return this.table;
    }

    @Override
    public BlockContinue tryContinue(ParserState state) {
        if (state.getLine().toString().contains("|")) {
            return BlockContinue.atIndex(state.getIndex());
        } else {
            return BlockContinue.none();
        }
    }

    @Override
    public void addLine(SourceLine line) {
        if (this.nextIsSeparatorLine) {
            this.nextIsSeparatorLine = false;
        } else {
            this.bodyLines.add(line.getContent());
        }
    }

    @Override
    public void parseInlines(InlineParser inlineParser) {
        var headerColumns = this.table.columnCount();

        var head = new Table.Head();
        this.table.appendChild(head);

        var headerRow = new Table.Row();
        head.appendChild(headerRow);
        for (int i = 0; i < headerColumns; i++) {
            var cell = this.table.getTitle(i);
            var tableCell = parseCell(cell, i, inlineParser);
            tableCell.setHeader(true);
            headerRow.appendChild(tableCell);
        }

        Node body = null;
        for (var rowLine : this.bodyLines) {
            var cells = TableParser.split(rowLine);
            var row = new Table.Row();

            // Body can not have more columns than head
            for (int i = 0; i < headerColumns; i++) {
                String cell = i < cells.size() ? cells.get(i) : "";
                Table.Cell tableCell = parseCell(cell, i, inlineParser);
                row.appendChild(tableCell);
            }

            if (body == null) {
                // It's valid to have a table without body. In that case, don't add an empty
                // TableBody node.
                body = new Table.Body();
                this.table.appendChild(body);
            }
            body.appendChild(row);
        }
    }

    private Table.Cell parseCell(String cell, int column, InlineParser inlineParser) {
        var tableCell = new Table.Cell();

        if (column < this.table.columnCount()) {
            tableCell.setWidth(this.table.getWidth(column));
        }
        if (column < this.table.columnCount()) {
            tableCell.setAlignment(this.table.getAlign(column));
        }

        var lines = new SourceLines();
        // Allow new lines
        lines.addLine(SourceLine.of(cell.trim().replace("\\\\", "\n"), null));
        inlineParser.parse(lines, tableCell);
        return tableCell;
    }

    private static List<String> split(CharSequence input) {
        var line = input.toString().trim();
        if (line.startsWith("|")) {
            line = line.substring(1);
        }
        var cells = new ArrayList<String>();
        var sb = new StringBuilder();
        for (int i = 0; i < line.length(); i++) {
            var c = line.charAt(i);
            switch (c) {
                case '\\':
                    if (((i + 1) < line.length()) && (line.charAt(i + 1) == '|')) {
                        // Pipe is special for table parsing. An escaped pipe doesn't result in a new
                        // cell, but
                        // is
                        // passed down to inline parsing as an unescaped pipe. Note that that applies
                        // even for
                        // the `\|`
                        // in an input like `\\|` - in other words, table parsing doesn't support
                        // escaping
                        // backslashes.
                        sb.append('|');
                        i++;
                    } else {
                        // Preserve backslash before other characters or at end of line.
                        sb.append('\\');
                    }
                    break;
                case '|':
                    cells.add(sb.toString());
                    sb.setLength(0);
                    break;
                default:
                    sb.append(c);
            }
        }
        if (!sb.isEmpty()) {
            cells.add(sb.toString());
        }
        return cells;
    }

    // Examples of valid separators:
    //
    // |-
    // -|
    // |-|
    // -|-
    // |-|-|
    // --- | ---
    private static void parseSeparator(CharSequence s, List<Integer> width,
                                       List<Table.Alignment> align) {
        int pipes = 0;
        int i = 0;
        int w = 0;
        while (i < s.length()) {
            char c = s.charAt(i);
            switch (c) {
                case '|':
                    i++;
                    pipes++;
                    if (pipes > 1) {
                        // More than one adjacent pipe isn't allowed
                        return;
                    }
                    // Need at least one pipe, even for a one-column table
                    break;
                case '-':
                case ':':
                    w = 0;
                    if ((pipes == 0) && !align.isEmpty()) {
                        // Need a pipe after the first column (first column doesn't need to start with
                        // one)
                        return;
                    }
                    boolean left = false;
                    boolean right = false;
                    if (c == ':') {
                        left = true;
                        i++;
                        w++;
                    }
                    boolean haveDash = false;
                    while ((i < s.length()) && (s.charAt(i) == '-')) {
                        i++;
                        w++;
                        haveDash = true;
                    }
                    if (!haveDash) {
                        // Need at least one dash
                        return;
                    }
                    if ((i < s.length()) && (s.charAt(i) == ':')) {
                        right = true;
                        i++;
                        w++;
                    }
                    width.add(w);
                    align.add(TableParser.getAlignment(left, right));
                    // Next, need another pipe
                    pipes = 0;
                    break;
                case ' ':
                case '\t':
                    // White space is allowed between pipes and columns
                    i++;
                    break;
                default:
                    // Any other character is invalid
                    return;
            }
        }
    }

    private static Table.Alignment getAlignment(boolean left, boolean right) {
        if (left && right) {
            return Table.Alignment.CENTER;
        } else if (left) {
            return Table.Alignment.LEFT;
        } else if (right) {
            return Table.Alignment.RIGHT;
        } else {
            return Table.Alignment.AUTO;
        }
    }

    public static class Factory extends AbstractBlockParserFactory {

        @Override
        public BlockStart tryStart(ParserState state, MatchedBlockParser matchedBlockParser) {
            var line = state.getLine().getContent();
            var paragraph = matchedBlockParser.getParagraphLines();
            if ((paragraph != null) && paragraph.getContent().contains("|") && !paragraph.toString()
                    .contains("\n")) {
                var separatorLine = line.subSequence(state.getIndex(), line.length());
                var width = new ArrayList<Integer>();
                var align = new ArrayList<Table.Alignment>();
                TableParser.parseSeparator(separatorLine, width, align);
                if (!align.isEmpty()) {
                    var title = TableParser.split(paragraph.getContent());
                    if (align.size() >= title.size()) {
                        return BlockStart.of(new TableParser(width, align, title))
                                .atIndex(state.getIndex())
                                .replaceActiveBlockParser();
                    }
                }
            }
            return BlockStart.none();
        }
    }
}
