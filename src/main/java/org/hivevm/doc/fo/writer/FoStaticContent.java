// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

/**
 * The {@link FoStaticContent} class.
 */
public class FoStaticContent extends FoNode {

    public FoStaticContent(FoPageSequence page) {
        super("static-content", page);
    }

    public final FoStaticContent setFlowName(String name) {
        set("flow-name", name);
        return this;
    }

    public FoBlock addBlock() {
        return FoBlock.block(this);
    }

    public FoBlockContainer blockContainer() {
        return new FoBlockContainer(this);
    }
}
