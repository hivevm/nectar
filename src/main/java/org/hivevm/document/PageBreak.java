// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

// Einfache Elemente
public class PageBreak implements DocumentElement {

    private final String id;

    private PageBreak(Builder builder) {
        this.id = builder.id;
    }

    @Override
    public String id() {
        return id;
    }

    @Override
    public <C> void accept(DocumentVisitor visitor, C context) {
        visitor.visit(this, context);
    }

    public static class Builder {

        private String id;

        public PageBreak build() {
            return new PageBreak(this);
        }
    }
}