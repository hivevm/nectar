// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

public class TextStyle {

    private final String color;
    private final boolean code;
    private final boolean bold;
    private final boolean italic;
    private final boolean overline;
    private final boolean underline;
    private final boolean strikethrough;

    private TextStyle(Builder builder) {
        this.code = builder.code;
        this.bold = builder.bold;
        this.color = builder.color;
        this.italic = builder.italic;
        this.overline = builder.overline;
        this.underline = builder.underline;
        this.strikethrough = builder.strikethrough;
    }

    public String color() {
        return color;
    }

    public boolean isCode() {
        return code;
    }

    public boolean isBold() {
        return bold;
    }

    public boolean isItalic() {
        return italic;
    }

    public boolean isOverline() {
        return overline;
    }

    public boolean isUnderline() {
        return underline;
    }

    public boolean isStrikethrough() {
        return strikethrough;
    }

    public static class Builder {

        private String color;
        private boolean code = false;
        private boolean bold = false;
        private boolean italic = false;
        private boolean overline = false;
        private boolean underline = false;
        private boolean strikethrough = false;

        public Builder bold() {
            this.bold = true;
            return this;
        }

        public Builder italic() {
            this.italic = true;
            return this;
        }

        public Builder overline() {
            this.overline = true;
            return this;
        }

        public Builder underline() {
            this.underline = true;
            return this;
        }

        public Builder strikethrough() {
            this.strikethrough = true;
            return this;
        }

        public Builder code() {
            this.code = true;
            return this;
        }

        public Builder color(String color) {
            this.color = color;
            return this;
        }

        public TextStyle build() {
            return new TextStyle(this);
        }
    }
}