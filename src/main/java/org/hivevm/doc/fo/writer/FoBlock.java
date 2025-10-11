// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

import org.hivevm.doc.fo.writer.set.*;

/**
 * The {@link FoBlock} class.
 */
public class FoBlock extends FoNode implements FoSpace<FoBlock>, FoMargin<FoBlock>,
        FoBorder<FoBlock>,
        FoPadding<FoBlock>, FoFont<FoBlock>, FoBreak<FoBlock>, FoBackground<FoBlock>,
        FoIndent<FoBlock> {

    /**
     * Constructs an instance of {@link FoBlock}.
     *
     * @param name
     * @param parent
     */
    public FoBlock(String name, FoNode parent) {
        super(name, parent);
    }

    public FoBlock setTextAlignLast(String align) {
        set("text-align-last", align);
        return this;
    }

    public FoBlock setKeepTogether(String keep) {
        set("keep-together", keep);
        return this;
    }

    public FoBlock setKeepWithNext(String keep) {
        set("keep-with-next.within-column", keep);
        return this;
    }

    public FoBlock setKeepWithPage(String keep) {
        set("keep-together.within-page", keep);
        return this;
    }

    public FoBlock setWarp(String value) {
        set("wrap-option", value);
        return this;
    }

    public FoBlock setWhiteSpaceCollapse(String value) {
        set("white-space-collapse", value);
        return this;
    }

    public FoBlock setWhiteSpaceTreatment(String value) {
        set("white-space-treatment", value);
        return this;
    }

    public FoBlock setLineFeed(String value) {
        set("linefeed-treatment", value);
        return this;
    }

    public FoBlock setPageBreak() {
        set("page-break-after", "always");
        return this;
    }


    public FoBlock setSpan(String value) {
        set("span", value);
        return this;
    }

    public FoBlock setWidth(String value) {
        set("width", value);
        return this;
    }

    public FoBlock setHeight(String value) {
        set("height", value);
        return this;
    }

    /**
     * Add a simple content.
     *
     * @param content
     */
    public FoBlock addContent(String content) {
        addText(content);
        return this;
    }

    public FoBlock addBlock() {
        return new FoBlock("block", this);
    }

    public FoBlock addInline() {
        return new FoBlock("inline", this);
    }

    public static FoBlock block(FoNode parent) {
        return new FoBlock("block", parent);
    }

    public static FoBlock inline(FoNode parent) {
        return new FoBlock("inline", parent);
    }
}
