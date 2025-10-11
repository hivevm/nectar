// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

/**
 * The {@link FoPageNumberCitation} class.
 */
public class FoPageNumberCitation extends FoNode {

    public FoPageNumberCitation(String id, FoNode parent) {
        super("page-number-citation", parent);
        set("ref-id", id);
    }
}
