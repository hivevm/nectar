// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

import org.hivevm.doc.fo.writer.set.FoBackground;
import org.hivevm.doc.fo.writer.set.FoBorder;
import org.hivevm.doc.fo.writer.set.FoSpace;

/**
 * The {@link FoTable} class.
 */
public class FoTable extends FoNode implements FoSpace<FoTable>, FoBorder<FoTable>,
        FoBackground<FoTable> {

    /**
     * Constructs an instance of {@link FoTable}.
     */
    public FoTable(FoNode parent) {
        super("table", parent);
    }

    public FoTable setTableLayout(String fixed) {
        set("table-layout", fixed);
        return this;
    }

    public FoTable setTableWidth(String width) {
        set("width", width);
        return this;
    }

    public FoTable setBorderBefore(String before) {
        set("border-before-width.conditionality", before);
        return this;
    }

    public FoTable setBorderCollapse(String before) {
        set("border-collapse", before);
        return this;
    }

    public FoTable setBorderSpacing(String spacing) {
        set("border-spacing", spacing);
        return this;
    }

    public FoTable addColumn(String number, String width) {
        FoNode builder = new FoNode("table-column", this);
        builder.set("column-number", number);
        builder.set("column-width", width);
        return this;
    }

    public FoTableArea addHead() {
        return addArea("table-header", "0pt", "0pt");
    }

    public FoTableArea addBody() {
        return addArea("table-body", "0pt", "0pt");
    }

    public FoTableArea addFoot() {
        return addArea("table-footer", "0pt", "0pt");
    }

    private FoTableArea addArea(String foName, String startIndent, String endIndent) {
        FoTableArea builder = new FoTableArea(foName, this);
        return builder.setStartIndent(startIndent).setEndIndentLastLine(endIndent);
    }
}
