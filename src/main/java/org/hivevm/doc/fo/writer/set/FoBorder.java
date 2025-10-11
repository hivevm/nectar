// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer.set;

/**
 * The {@link FoBorder} class.
 */
public interface FoBorder<F extends FoBorder<?>> extends Fo {

    @SuppressWarnings("unchecked")
    default F setBorder(String width, String style, String color) {
        setBorderTop(width, style, color);
        setBorderLeft(width, style, color);
        setBorderRight(width, style, color);
        setBorderBottom(width, style, color);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setBorderWidth(String borderWidth) {
        set("border-width", borderWidth);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setBorderTop(String width, String style, String color) {
        set("border-top-width", width);
        set("border-top-style", style);
        set("border-top-color", color);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setBorderLeft(String width, String style, String color) {
        set("border-start-width", width);
        set("border-start-style", style);
        set("border-start-color", color);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setBorderRight(String width, String style, String color) {
        set("border-end-width", width);
        set("border-end-style", style);
        set("border-end-color", color);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setBorderBottom(String width, String style, String color) {
        set("border-bottom-width", width);
        set("border-bottom-style", style);
        set("border-bottom-color", color);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setBorderRadius(String radius) {
        set("fox:border-radius", radius);
        return (F) this;
    }

}
