// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

import org.hivevm.util.xml.XmlBuilder;

/**
 * The {@link FoRoot} class.
 */
public class FoRoot extends FoNode {

    /**
     * Constructs an instance of {@link FoRoot}.
     */
    public FoRoot(XmlBuilder builder) {
        super("root", builder);
        set("font-selection-strategy", "character-by-character");
        set("text-align", "justify").set("line-height", "1.4em");
        set("line-height-shift-adjustment", "disregard-shifts");
        set("writing-mode", "lr-tb").set("language", "en");
    }

    public FoRoot setFontFamily(String name) {
        set("font-family", name);
        return this;
    }

    public FoRoot setFontSize(String value) {
        set("font-size", value);
        return this;
    }

    /**
     * Get the {@link FoNode} for the layout master set.
     */
    public final FoLayoutMasterSet createLayoutMasteSet() {
        return new FoLayoutMasterSet(this);
    }

    public FoPageSequence addPageSequence(String id) {
        var seq = new FoPageSequence(this);
        seq.setId(id);
        return seq;
    }
}
