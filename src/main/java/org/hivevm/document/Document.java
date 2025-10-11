// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

public class Document {

    private final String title;
    private final String subTitle;
    private final List<DocumentElement> elements;

    private Document(Builder builder) {
        this.title = builder.title;
        this.subTitle = builder.subTitle;
        this.elements = new ArrayList<>(builder.elements);
    }

    public DocumentElement getElementById(String id) {
        return elements.stream()
                .filter(element -> element.id().equals(id))
                .findFirst()
                .orElse(null);
    }

    public String title() {
        return title;
    }

    public String subTitle() {
        return subTitle;
    }

    public List<DocumentElement> elements() {
        return Collections.unmodifiableList(elements);
    }

    public static class Builder implements ContainerElement.Builder {

        private String title;
        private String subTitle;
        private List<DocumentElement> elements = new ArrayList<>();
        private int footnotes;

        public Builder setTitle(String title) {
            this.title = title;
            return this;
        }

        public Builder setSubTitle(String subTitle) {
            this.subTitle = subTitle;
            return this;
        }

        public PageBreak.Builder addPageBreak() {
            return new PageBreak.Builder() {

                @Override
                public PageBreak build() {
                    var element = super.build();
                    elements.add(element);
                    return element;
                }
            };
        }

        @Override
        public Paragraph.Builder addParagraph() {
            return new Paragraph.Builder() {

                @Override
                public Paragraph build() {
                    var element = super.build();
                    elements.add(element);
                    return element;
                }
            };
        }

        public Heading.Builder addHeader(int level) {
            return new Heading.Builder(level) {

                @Override
                public Heading build() {
                    var element = super.build();
                    elements.add(element);
                    return element;
                }
            };
        }

        @Override
        public CodeBlock.Builder addCodeBlock(String code, String language) {
            return new CodeBlock.Builder(null, code) {

                @Override
                public DocumentElement build() {
                    var element = super.build();
                    elements.add(element);
                    return element;
                }
            }.language(language);
        }

        @Override
        public Block.Builder addBlock(String cssClass) {
            return new Block.Builder(null) {

                @Override
                public Block build() {
                    var element = super.build();
                    elements.add(element);
                    return element;
                }
            }.cssClass(cssClass);
        }

        public ListElement.Builder addList(ListElement.ListType type) {
            return new ListElement.Builder(null, type) {

                @Override
                public ListElement build() {
                    var element = super.build();
                    elements.add(element);
                    return element;
                }
            };
        }

        public Table.Builder addTable() {
            return new Table.Builder(null) {

                @Override
                public Table build() {
                    var element = super.build();
                    elements.add(element);
                    return element;
                }
            };
        }

        public int nextFootnote() {
            return ++this.footnotes;
        }

        public Document build() {
            return new Document(this);
        }
    }
}