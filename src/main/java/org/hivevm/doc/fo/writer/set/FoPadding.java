// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer.set;

/**
 * The {@link FoPadding} class.
 */
public interface FoPadding<F extends FoPadding<?>> extends Fo {

    @SuppressWarnings("unchecked")
    default F setPadding(String value) {
        set("padding", value);
        return (F) this;
    }

    default F setPadding(String vertical, String horizontal) {
        return setPadding(horizontal, horizontal, vertical, vertical);
    }

    @SuppressWarnings("unchecked")
    default F setPaddingLeftRight(String value) {
        set("padding-left", value);
        set("padding-right", value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setPadding(String left, String right, String top, String bottom) {
        setPaddingTop(top);
        setPaddingLeft(left);
        setPaddingRight(right);
        setPaddingBottom(bottom);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setPaddingTop(String value) {
        set("padding-top", value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setPaddingLeft(String value) {
        set("padding-left", value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setPaddingRight(String value) {
        set("padding-right", value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setPaddingBottom(String value) {
        set("padding-bottom", value);
        return (F) this;
    }
}
