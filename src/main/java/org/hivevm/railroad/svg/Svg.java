// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.svg;

import org.hivevm.railroad.Color;
import org.hivevm.railroad.Font;

/**
 * The {@link Svg} implements utilities for SVG.
 */
public interface Svg {

    /**
     * Escapes the XML string
     */
    static String escapeXML(String xml) {
        if (xml == null || xml.isEmpty())
            return xml;

        StringBuilder builder = new StringBuilder((int) (xml.length() * 1.1));
        for (var c : xml.toCharArray()) {
            var text = switch (c) {
                case '<' -> "&lt;";
                case '>' -> "&gt;";
                case '&' -> "&amp;";
                case '\'' -> "&apos;";
                case '\"' -> "&quot;";
                default -> c;
            };
            builder.append(text);
        }
        return builder.toString();
    }

    /**
     * Convert a {@link Color} to an CSS string.
     */
    static String toString(Color c) {
        StringBuilder builder = new StringBuilder("#");
        if (c.red() < 16)
            builder.append('0');
        builder.append(Integer.toHexString(c.red()));
        if (c.green() < 16)
            builder.append('0');
        builder.append(Integer.toHexString(c.green()));
        if (c.blue() < 16)
            builder.append('0');
        builder.append(Integer.toHexString(c.blue()));
        return builder.toString();
    }

    /**
     * Convert a {@link Font} to an CSS string.
     */
    static String toString(Font font) {
        StringBuilder builder = new StringBuilder();
        builder.append("font-family:").append(font.name()).append(",Sans-serif;");
        if (font.isItalic())
            builder.append("font-style:italic;");
        if (font.isBold())
            builder.append("font-weight:bold;");
        builder.append("font-size:").append(font.size()).append("px;");
        return builder.toString();
    }
}
