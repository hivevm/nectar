// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

// Einfache Elemente
public class Paragraph implements DocumentElement {

    private final String id;
    private final List<TextSpan> textSpans;

    private Paragraph(Builder builder) {
        this.id = builder.id;
        this.textSpans = Collections.unmodifiableList(builder.textSpans);
    }

    @Override
    public String id() {
        return id;
    }

    public List<TextSpan> textSpans() {
        return textSpans;
    }

    @Override
    public <C> void accept(DocumentVisitor visitor, C context) {
        visitor.visit(this, context);
    }

    public static class Builder implements TextBuilder<Builder> {

        private String id;
        private List<TextSpan> textSpans = new ArrayList<>();

        public Builder addText(TextSpan textSpan) {
            this.textSpans.add(textSpan);
            return this;
        }

        public Paragraph build() {
            return new Paragraph(this);
        }
    }
}