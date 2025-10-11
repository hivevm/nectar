// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.commonmark.image;

import java.util.Map;
import org.commonmark.node.CustomNode;
import org.commonmark.node.Delimited;

/**
 * The ImageAttributes class represents a custom node used to store image attributes defined within
 * a custom CommonMark syntax. This class allows attributes to be parsed and associated with a
 * corresponding image node in a CommonMark document.
 * <p>
 * This class is particularly utilized in conjunction with the custom delimiter processor and
 * attribute providers as defined in the wider API to extend the CommonMark rendering and parsing
 * capabilities.
 * <p>
 * It implements the CustomNode for compatibility with the CommonMark Node hierarchy and the
 * Delimited interface to provide clear handling of its opening and closing delimiters.
 */
public class ImageAttributes extends CustomNode implements Delimited {

    private final Map<String, String> attributes;

    public ImageAttributes(Map<String, String> attributes) {
        this.attributes = attributes;
    }

    @Override
    public String getOpeningDelimiter() {
        return "{";
    }

    @Override
    public String getClosingDelimiter() {
        return "}";
    }

    public Map<String, String> getAttributes() {
        return this.attributes;
    }

    @Override
    protected String toStringAttributes() {
        return "imageAttributes=" + this.attributes;
    }
}
