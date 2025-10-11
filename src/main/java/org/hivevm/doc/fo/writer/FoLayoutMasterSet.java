// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

/**
 * The {@link FoLayoutMasterSet} class.
 */
public class FoLayoutMasterSet extends FoNode {

    public FoLayoutMasterSet(FoNode parent) {
        super("layout-master-set", parent);
    }

    /**
     * Get the {@link FoNode} for the layout master set.
     */
    public final FoSimplePageMaster createPageMaster(String name) {
        return new FoSimplePageMaster(name, this);
    }

    /**
     * Get the {@link FoNode} for the layout master set.
     */
    public final FoPageSequenceMaster createPageSequence(String name) {
        return new FoPageSequenceMaster(name, this);
    }
}
