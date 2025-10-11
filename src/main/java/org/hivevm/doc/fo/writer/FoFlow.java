// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

import org.hivevm.doc.fo.writer.set.FoIndent;

/**
 * The {@link FoFlow} class.
 */
public class FoFlow extends FoNode implements FoIndent<FoFlow> {

    /**
     * Constructs an instance of {@link FoFlow}.
     */
    public FoFlow(FoPageSequence sequence) {
        super("flow", sequence);
    }

    /**
     * Constructs an instance of {@link FoFlow}.
     *
     * @param name
     */
    public FoFlow setName(String name) {
        set("flow-name", name);
        return this;
    }
}
