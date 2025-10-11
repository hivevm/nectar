// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.commonmark.marker;

import org.commonmark.node.CustomNode;

/**
 * A strikethrough node containing text and other inline nodes nodes as children.
 */
public class Marker extends CustomNode {

    public enum Decoration {
        Overline,
        Underline,
        Highlight,
        Strikethrough
    }

    private final Decoration decoration;

    /**
     * Constructs an instance of {@link Marker}.
     *
     * @param decoration
     */
    protected Marker(Decoration decoration) {
        this.decoration = decoration;
    }

    /**
     * Gets the {@link Decoration}..
     */
    public final Decoration getDecoration() {
        return this.decoration;
    }
}
