// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

/**
 * The {@link FoBookmark} class.
 */
public class FoBookmark extends FoNode {

    private final FoNode title;

    /**
     * Constructs an instance of {@link FoBookmark}.
     *
     * @param id
     * @param bookmark
     */
    public FoBookmark(String id, FoNode bookmark) {
        super("bookmark", bookmark);
        set("internal-destination", id);
        set("starting-state", "hide");

        this.title = new FoNode("bookmark-title", this);
    }

    /**
     * Set the title of the bookmark.
     *
     * @param title
     */
    public FoBookmark setTitle(String title) {
        this.title.addText(title);
        return this;
    }

    /**
     * Add a child bookmark.
     *
     * @param id
     */
    public FoBookmark addBookmark(String id) {
        return new FoBookmark(id, this);
    }
}
