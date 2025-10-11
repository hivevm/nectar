// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

import org.hivevm.doc.fo.writer.set.FoBackground;
import org.hivevm.doc.fo.writer.set.FoBorder;
import org.hivevm.doc.fo.writer.set.FoFont;

/**
 * The {@link FoTableRow} class.
 */
public class FoTableRow extends FoNode implements FoBorder<FoTableRow>, FoFont<FoTableRow>,
        FoBackground<FoTableRow> {

    /**
     * Constructs an instance of {@link FoTableRow}.
     */
    FoTableRow(FoTableArea parent) {
        super("table-row", parent);
    }

    public FoTableRow setDisplayAlign(String align) {
        set("display-align", align);
        return this;
    }

    public FoTableRow setProgessionDimensionMin(String value) {
        set("block-progression-dimension.minimum", value);
        return this;
    }

    public FoTableCell addCell() {
        return new FoTableCell(this);
    }
}
