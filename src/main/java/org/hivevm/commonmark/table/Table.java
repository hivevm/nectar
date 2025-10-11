// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.commonmark.table;

import org.commonmark.node.CustomBlock;
import org.commonmark.node.CustomNode;

import java.util.List;

/**
 * Represents a table structure in a document. A table consists of a head, body, rows, and cells,
 * and supports alignment and width definitions for individual cells.
 */
public class Table extends CustomBlock {

    public enum Alignment {
        AUTO,
        LEFT,
        CENTER,
        RIGHT
    }

    private final List<Integer> columnWidth;
    private final List<Table.Alignment> columnAlign;
    private final List<String> columnTitle;

    public Table(List<Integer> columnWidth, List<Table.Alignment> columnAlign, List<String> columnTitle) {
        this.columnWidth = columnWidth;
        this.columnAlign = columnAlign;
        this.columnTitle = columnTitle;
    }

    public int columnCount() {
        return columnWidth.size();
    }

    public int getWidth(int index) {
        return columnWidth.get(index);
    }

    public Alignment getAlign(int index) {
        return columnAlign.get(index);
    }

    public String getTitle(int index) {
        return columnTitle.get(index);
    }

    public static class Body extends CustomNode {

    }

    public static class Head extends CustomNode {

    }

    public static class Row extends CustomNode {

    }

    public static class Cell extends CustomNode {

        private boolean header;
        private int width;
        private Alignment alignment;

        public boolean isHeader() {
            return this.header;
        }

        public void setHeader(boolean header) {
            this.header = header;
        }

        public int getWidth() {
            return this.width == 0 ? 1 : this.width;
        }

        public void setWidth(int width) {
            this.width = width;
        }

        public Alignment getAlignment() {
            return this.alignment == null ? Alignment.LEFT : this.alignment;
        }

        public void setAlignment(Alignment alignment) {
            this.alignment = alignment;
        }
    }
}
