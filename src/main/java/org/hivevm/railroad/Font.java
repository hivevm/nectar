// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad;

import java.awt.font.FontRenderContext;

public record Font(String name, int size, boolean isBold, boolean isItalic) {

    private static final FontRenderContext CONTEXT = new FontRenderContext(null, true, false);

    public Font(String name, int size) {
        this(name, size, false, false);
    }

    private java.awt.Font asAwtFont() {
        var style = 0;
        if (isBold)
            style += java.awt.Font.BOLD;
        if (isItalic)
            style += java.awt.Font.ITALIC;
        return new java.awt.Font(name, style, size);
    }


    public int getFontWidth(String text) {
        var bounds = asAwtFont().getStringBounds(text, Font.CONTEXT);
        return (int) Math.round(bounds.getWidth());
    }

    public int getFontHeight(String text) {
        var bounds = asAwtFont().getStringBounds(text, Font.CONTEXT);
        return (int) Math.round(bounds.getHeight());
    }

    public int getLineMetrics(String text) {
        var lineMetrics = asAwtFont().getLineMetrics(text, Font.CONTEXT);
        return Math.round(lineMetrics.getDescent());
    }
}
