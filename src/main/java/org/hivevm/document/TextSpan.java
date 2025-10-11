// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

public class TextSpan {

    private final String text;
    private final String scope;
    private final TextStyle style;
    private final boolean end;
    private final boolean code;
    private final boolean bold;
    private final boolean italic;
    private final boolean overline;
    private final boolean underline;
    private final boolean strikethrough;
    private final boolean newLine;

    private final String linkUrl;
    private final String linkIndex;
    private final String linkTitle;

    private String imageUri;
    private String imageTitle;
    private String imageAlign;
    private String imageWidth;
    private String imageHeight;

    private TextSpan(Builder builder) {
        this.text = builder.text;
        this.end = builder.end;
        this.code = builder.code;
        this.bold = builder.bold;
        this.scope = builder.scope;
        this.style = builder.style;
        this.italic = builder.italic;
        this.overline = builder.overline;
        this.underline = builder.underline;
        this.strikethrough = builder.strikethrough;
        this.newLine = builder.newLine;

        this.linkUrl = builder.linkUrl;
        this.linkIndex = builder.linkIndex;
        this.linkTitle = builder.linkTitle;

        this.imageUri = builder.imageUri;
        this.imageTitle = builder.imageTitle;
        this.imageAlign = builder.imageAlign;
        this.imageWidth = builder.imageHeight;
        this.imageHeight = builder.imageHeight;
    }

    // Getters
    public String text() {
        return text;
    }

    public String scope() {
        return scope;
    }

    public TextStyle style() {
        return style;
    }

    public boolean isEnd() {
        return end;
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

    public boolean isNewLine() {
        return newLine;
    }

    public boolean isLink() {
        return linkUrl != null;
    }

    public String getLinkUrl() {
        return linkUrl;
    }

    public String getLinkIndex() {
        return linkIndex;
    }

    public String getLinkTitle() {
        return linkTitle;
    }

    public boolean isImage() {
        return imageUri != null;
    }

    public String getImageUri() {
        return imageUri;
    }

    public String getImageAlign() {
        return imageAlign;
    }

    public String getImageTitle() {
        return imageTitle;
    }

    public String getImageWidth() {
        return imageWidth;
    }

    public String getImageHeight() {
        return imageHeight;
    }

    public static class Builder {

        private String text;
        private String scope;
        private TextStyle style;
        private boolean end = false;
        private boolean code = false;
        private boolean bold = false;
        private boolean italic = false;
        private boolean overline = false;
        private boolean underline = false;
        private boolean strikethrough = false;
        private boolean newLine;

        private String linkUrl;
        private String linkIndex;
        private String linkTitle;

        private String imageUri;
        private String imageTitle;
        private String imageAlign;
        private String imageWidth;
        private String imageHeight;

        public Builder() {
            this.text = "";
        }

        public Builder(String text) {
            this.text = text;
        }

        public Builder end() {
            this.end = true;
            return this;
        }

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

        public Builder scope(String scope) {
            this.scope = scope;
            return this;
        }

        public Builder style(TextStyle style) {
            this.style = style;
            return this;
        }

        public Builder newLine() {
            this.newLine = true;
            return this;
        }

        public Builder link(String url, String title, Document.Builder document) {
            this.linkUrl = url;
            this.linkIndex = title != null ? "" + document.nextFootnote() : null;
            this.linkTitle = title;
            return this;
        }

        public Builder image(String uri, String title, String align, String width, String height) {
            this.imageUri = uri;
            this.imageTitle = title;
            this.imageAlign = align;
            this.imageWidth = width;
            this.imageHeight = height;
            return this;
        }

        public TextSpan build() {
            return new TextSpan(this);
        }
    }
}