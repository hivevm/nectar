// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer.set;

/**
 * The {@link FoBreak} class.
 */
public interface FoBreak<F extends FoBreak<?>> extends Fo {

    @SuppressWarnings("unchecked")
    default F setBreakBefore(String value) {
        set("break-before", value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setBreakAfter(String value) {
        set("break-after", value);
        return (F) this;
    }
}
