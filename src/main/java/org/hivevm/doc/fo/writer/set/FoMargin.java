// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer.set;

/**
 * The {@link FoMargin} class.
 */
public interface FoMargin<F extends FoMargin<?>> extends Fo {

    @SuppressWarnings("unchecked")
    default F setMargin(String value) {
        set("margin", value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setMarginTop(String value) {
        set("margin-top", value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setMarginLeft(String value) {
        set("margin-left", value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setMarginRight(String value) {
        set("margin-right", value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setMarginBottom(String value) {
        set("margin-bottom", value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setMargin(String left, String right, String top, String bottom) {
        setMarginTop(top);
        setMarginLeft(left);
        setMarginRight(right);
        setMarginBottom(bottom);
        return (F) this;
    }
}
