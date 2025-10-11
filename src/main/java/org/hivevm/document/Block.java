// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

import java.util.*;

public class Block implements ContainerElement {

    private final String id;
    private final List<DocumentElement> children;
    private final String cssClass;
    private final Map<String, String> attributes;

    private Block(Builder builder) {
        this.id = builder.id;
        this.children = new ArrayList<>(builder.children);
        this.cssClass = builder.cssClass;
        this.attributes = Collections.unmodifiableMap(builder.attributes);
    }

    @Override
    public <C> void accept(DocumentVisitor visitor, C context) {
        visitor.visit(this, context);
    }

    @Override
    public String id() {
        return id;
    }

    @Override
    public List<DocumentElement> getChildren() {
        return Collections.unmodifiableList(children);
    }

    public String getCssClass() {
        return cssClass;
    }

    public Map<String, String> getAttributes() {
        return attributes;
    }

    public static class Builder implements ContainerElement.Builder {

        private String id;
        private List<DocumentElement> children = new ArrayList<>();
        private String cssClass = "";
        private Map<String, String> attributes = new HashMap<>();

        public Builder(String id) {
            this.id = id;
        }

        public Builder cssClass(String cssClass) {
            this.cssClass = cssClass;
            return this;
        }

        public Builder attribute(String key, String value) {
            this.attributes.put(key, value);
            return this;
        }

        @Override
        public Paragraph.Builder addParagraph() {
            return new Paragraph.Builder() {

                @Override
                public Paragraph build() {
                    Paragraph paragraph = super.build();
                    children.add(paragraph);
                    return paragraph;
                }
            };
        }

        @Override
        public CodeBlock.Builder addCodeBlock(String code, String language) {
            return new CodeBlock.Builder(null, code) {

                @Override
                public DocumentElement build() {
                    var heading = super.build();
                    children.add(heading);
                    return heading;
                }
            }.language(language);
        }

        @Override
        public Block.Builder addBlock(String cssClass) {
            return new Block.Builder(null) {

                @Override
                public Block build() {
                    Block heading = super.build();
                    children.add(heading);
                    return heading;
                }
            }.cssClass(cssClass);
        }

        public ListElement.Builder addList(ListElement.ListType type) {
            return new ListElement.Builder(null, type) {

                @Override
                public ListElement build() {
                    ListElement heading = super.build();
                    children.add(heading);
                    return heading;
                }
            };
        }

        public Block build() {
            return new Block(this);
        }
    }
}