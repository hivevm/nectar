// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

/**
 * The {@link FoPageSequence} class.
 */
public class FoPageSequence extends FoNode {

    /**
     * Constructs an instance of {@link FoPageSequence}.
     */
    public FoPageSequence(FoRoot root) {
        super("page-sequence", root);
    }

    public FoPageSequence setReference(String reference) {
        set("master-reference", reference);
        return this;
    }

    public FoPageSequence setLanguage(String language) {
        set("language", language);
        return this;
    }

    public FoPageSequence setFormat(String format) {
        set("format", format);
        return this;
    }

    public FoPageSequence setInitialPageNumber(String initial) {
        set("initial-page-number", initial);
        return this;
    }

    public FoPageSequence setForcePageCount(String force) {
        set("force-page-count", force);
        return this;
    }

    public FoFlow flow() {
        return new FoFlow(this);
    }
}
