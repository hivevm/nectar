// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer.set;

/**
 * The {@link FoSpace} class.
 */
public interface FoSpace<F extends FoSpace<?>> extends Fo {

    @SuppressWarnings("unchecked")
    default F setSpace(String value) {
        setSpaceAfter(value);
        setSpaceBefore(value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setSpaceBefore(String value) {
        set("space-before", value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setSpaceAfter(String value) {
        set("space-after", value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setSpaceBefore(String minimum, String optimum, String maximum) {
        set("space-before.minimum", minimum);
        set("space-before.optimum", optimum);
        set("space-before.maximum", maximum);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setSpaceAfter(String minimum, String optimum, String maximum) {
        set("space-after.minimum", minimum);
        set("space-after.optimum", optimum);
        set("space-after.maximum", maximum);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setSpace(String minimum, String optimum, String maximum) {
        setSpaceAfter(minimum, optimum, maximum);
        setSpaceBefore(minimum, optimum, maximum);
        return (F) this;
    }
}
