// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

import org.hivevm.doc.fo.writer.set.FoFont;

/**
 * The {@link FoFootnote} class.
 */
public class FoFootnote extends FoNode implements FoFont<FoFootnote> {

    private final FoBlock body;

    /**
     * Constructs an instance of {@link FoFootnote}.
     */
    public FoFootnote(String id, FoNode node) {
        super("footnote", node);

        FoBlock inline = FoBlock.inline(this);
        inline.set("baseline-shift", "super").set("font-size", "smaller");
        inline.addText(id);

        FoNode content = new FoNode("footnote-body", this);
        this.body = FoBlock.block(content);

        inline = this.body.addInline();
        inline.set("baseline-shift", "super").set("font-size", "smaller");
        inline.addText(id);
    }

    /**
     * Get the footnote body.
     */
    public FoBlock getBody() {
        return this.body;
    }
}
