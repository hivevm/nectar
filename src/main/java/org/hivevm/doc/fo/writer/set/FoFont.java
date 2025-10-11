// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer.set;

/**
 * The {@link FoFont} class.
 */
public interface FoFont<F extends FoFont<?>> extends Fo {

    @SuppressWarnings("unchecked")
    default F setColor(String color) {
        set("color", color);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setFontFamily(String family) {
        set("font-family", family);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setFontWeight(String weight) {
        set("font-weight", weight);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setFontStyle(String style) {
        set("font-style", style);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setFontSize(String size) {
        set("font-size", size);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setLineHeight(String height) {
        set("line-height", height);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setTextAlign(String align) {
        set("text-align", align);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setTextDecoration(String value) {
        set("text-decoration", value);
        return (F) this;
    }
}
