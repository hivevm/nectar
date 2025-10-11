// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.stream.Collectors;

public class Table implements ContainerElement {

    private final String id;
    private final List<TableRow> rows;
    private final List<Column> columns;

    public enum RowType {HEAD, BODY, TAIL}

    public record Column(int index, int width, String align) {
    }

    private Table(Builder builder) {
        this.id = builder.id;
        this.rows = Collections.unmodifiableList(builder.rows);
        this.columns = Collections.unmodifiableList(builder.columns);
    }

    @Override
    public <C> void accept(DocumentVisitor visitor, C context) {
        visitor.visit(this, context);
    }

    @Override
    public String id() {
        return id;
    }

    @Override
    public List<DocumentElement> getChildren() {
        return rows.stream()
                .map(DocumentElement.class::cast)
                .collect(Collectors.toList());
    }

    public List<Column> getColumns() {
        return columns;
    }

    public int getColumnWidth(int column) {
        var width = new ArrayList<Integer>();
        if (!this.columns.isEmpty()) {
            int count = 0;
            int sum = this.columns.stream()
                    .collect(Collectors.summingInt(c -> c.width()));
            for (int index = 0; index < this.columns.size(); index++) {
                int size = (int) ((this.columns.get(index).width() * 100f) / sum);
                count += size;
                if ((index + 1) == this.columns.size()) {
                    size += 100 - count;
                }
                width.add(size);
            }
        }
        return width.get(column);
    }

    public List<TableRow> getRows() {
        return rows;
    }

    public static class Builder {

        private String id;
        private List<TableRow> rows = new ArrayList<>();
        private List<Column> columns = new ArrayList<>();

        public Builder(String id) {
            this.id = id;
        }

        public Builder addColumn(int index, int width, String align) {
            this.columns.add(new Column(index, width, align));
            return this;
        }

        public Builder addRow(RowType rowType) {
            this.rows.add(new TableRow(null, rowType, new ArrayList<>()));
            return this;
        }

        public TableCell.Builder addCell() {
            return new TableCell.Builder() {

                @Override
                public TableCell build() {
                    var element = super.build();
                    rows.get(rows.size() - 1).cells.add(element);
                    return element;
                }
            };
        }

        public Table build() {
            return new Table(this);
        }
    }

    // Innere Klasse für Tabellenzeilen
    public static class TableRow implements DocumentElement {

        private final String id;
        private final RowType rowType;
        private final List<TableCell> cells;

        public TableRow(String id, RowType rowType, List<TableCell> cells) {
            this.id = id;
            this.rowType = rowType;
            this.cells = new ArrayList<>(cells);
        }

        @Override
        public <C> void accept(DocumentVisitor visitor, C context) {
            // TableRow wird durch Table-Visitor behandelt
        }

        @Override
        public String id() {
            return id;
        }

        public RowType rowType() {
            return rowType;
        }

        public List<TableCell> getCells() {
            return Collections.unmodifiableList(cells);
        }
    }

    public static class TableCell {

        private final List<TextSpan> content;
        private final int colspan;
        private final int rowspan;

        public TableCell(TableCell.Builder builder) {
            this.content = new ArrayList<>(builder.textSpans);
            this.colspan = 1;
            this.rowspan = 1;
        }

        public List<TextSpan> getContent() {
            return Collections.unmodifiableList(content);
        }

        public int getColspan() {
            return colspan;
        }

        public int getRowspan() {
            return rowspan;
        }

        public static class Builder implements TextBuilder<TableCell.Builder> {

            private String id;
            private List<TextSpan> textSpans = new ArrayList<>();

            public TableCell.Builder addText(TextSpan textSpan) {
                this.textSpans.add(textSpan);
                return this;
            }

            public TableCell build() {
                return new TableCell(this);
            }
        }
    }
}