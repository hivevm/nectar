// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

public class Image implements DocumentElement {

    private final String id;
    private final String src;
    private final String altText;
    private final String caption;
    private final int width;
    private final int height;

    private Image(Builder builder) {
        this.id = builder.id;
        this.src = builder.src;
        this.altText = builder.altText;
        this.caption = builder.caption;
        this.width = builder.width;
        this.height = builder.height;
    }

    protected Image(String id, String src) {
        this.id = id;
        this.src = src;
        this.altText = null;
        this.caption = null;
        this.width = 0;
        this.height = 0;
    }

    @Override
    public String id() {
        return id;
    }

    // Getters
    public String getSrc() {
        return src;
    }

    public String getAltText() {
        return altText;
    }

    public String getCaption() {
        return caption;
    }

    public int getWidth() {
        return width;
    }

    public int getHeight() {
        return height;
    }

    @Override
    public <C> void accept(DocumentVisitor visitor, C context) {
        visitor.visit(this, context);
    }

    public static class Builder {

        private String id;
        private String src;
        private String altText = "";
        private String caption = "";
        private int width = 0;
        private int height = 0;

        public Builder(String id, String src) {
            this.id = id;
            this.src = src;
        }

        public Builder altText(String altText) {
            this.altText = altText;
            return this;
        }

        public Builder caption(String caption) {
            this.caption = caption;
            return this;
        }

        public Builder dimensions(int width, int height) {
            this.width = width;
            this.height = height;
            return this;
        }

        public Image build() {
            return new Image(this);
        }
    }
}