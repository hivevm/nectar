// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Set;

/**
 * The {@link PageSet} defines a set of {@link Page}'s.
 */
public class PageSet {

    private final String               name;
    private final Map<Page, PageMatch> pages = new LinkedHashMap<>();

    /**
     * Constructs an instance of {@link PageSet}.
     */
    public PageSet(String name) {
        this.name = name;
    }

    public String getName() {
        return name;
    }

    /**
     * Adds a {@link Page}.
     */
    public final void addPage(Page page, PageMatch match) {
        this.pages.put(page, match);
    }

    public final Set<Entry<Page, PageMatch>> pages() {
        return this.pages.entrySet();
    }
}
