// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

public class Footnote {

    private final String id;
    private final List<TextSpan> content;

    public Footnote(String id, List<TextSpan> content) {
        this.id = id;
        this.content = new ArrayList<>(content);
    }

    public String getId() {
        return id;
    }

    public List<TextSpan> getContent() {
        return Collections.unmodifiableList(content);
    }
}