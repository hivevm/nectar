// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

// Header Elemente
public class Heading implements DocumentElement {

    private final String id;
    private final int level;
    private final String title;
    private final List<TextSpan> textSpans;

    private Heading(Builder builder) {
        this.id = builder.id;
        this.level = builder.level;
        this.title = builder.title;
        this.textSpans = Collections.unmodifiableList(builder.textSpans);
    }

    @Override
    public String id() {
        return id != null ? id : title.replace(' ', '_');
    }

    public int level() {
        return level;
    }

    public String title() {
        return title;
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
        private int level;
        private String title;
        private List<TextSpan> textSpans = new ArrayList<>();

        public Builder(int level) {
            this.level = level;
        }

        public Builder addText(TextSpan textSpan) {
            this.textSpans.add(textSpan);
            return this;
        }

        public Heading build() {
            StringBuilder builder = new StringBuilder();
            for (var textSpan : textSpans) {
                builder.append(textSpan.text());
            }
            this.title = builder.toString();
            return new Heading(this);
        }
    }
}