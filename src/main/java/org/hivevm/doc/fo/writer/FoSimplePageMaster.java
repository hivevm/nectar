// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

import org.hivevm.doc.fo.writer.set.FoMargin;

/**
 * The {@link FoSimplePageMaster} class.
 */
public class FoSimplePageMaster extends FoNode implements FoMargin<FoSimplePageMaster> {

    /**
     * Constructs an instance of {@link FoSimplePageMaster}.
     */
    FoSimplePageMaster(String name, FoNode layout) {
        super("simple-page-master", layout);
        set("master-name", name);
        setPageSize("210mm", "297mm");
    }

    public FoSimplePageMaster setPageSize(String width, String height) {
        setPageWidth(width);
        setPageHeight(height);
        return this;
    }

    public void setPageWidth(String width) {
        set("page-width", width);
    }

    public void setPageHeight(String height) {
        set("page-height", height);
    }

    public FoRegion createBodyRegion(String name) {
        return new FoRegion(this).setRegionName(name);
    }

    public FoRegion createRegionBefore(String name) {
        return new FoRegion(name, "before", this);
    }

    public FoRegion createRegionAfter(String name) {
        return new FoRegion(name, "after", this);
    }

    public FoRegion createRegionStart(String name) {
        return new FoRegion(name, "start", this);
    }

    public FoRegion createRegionEnd(String name) {
        return new FoRegion(name, "end", this);
    }
}
