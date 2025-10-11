// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.stream.Collectors;

public class ListElement implements ContainerElement {

    public enum ListType {ORDERED, UNORDERED}

    public enum ListMarker {NONE, CHECKED, UNCHECKED}

    private final String id;
    private final ListType type;
    private final List<ListItem> items;

    private ListElement(Builder builder) {
        this.id = builder.id;
        this.type = builder.type;
        this.items = new ArrayList<>(builder.items);
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
        return items.stream()
                .map(DocumentElement.class::cast)
                .collect(Collectors.toList());
    }

    public ListType getType() {
        return type;
    }

    public boolean isOrdered() {
        return type == ListType.ORDERED;
    }

    public List<ListItem> getItems() {
        return Collections.unmodifiableList(items);
    }

    public static class Builder {

        private String id;
        private ListType type;
        private List<ListItem> items = new ArrayList<>();

        public Builder(String id, ListType type) {
            this.id = id;
            this.type = type;
        }

        public ListItem.Builder addItem() {
            return new ListItem.Builder(null) {

                @Override
                public ListItem build() {
                    ListItem heading = super.build();
                    items.add(heading);
                    return heading;
                }
            };
        }

        public ListElement build() {
            return new ListElement(this);
        }
    }

    public static class ListItem implements ContainerElement {

        private final String id;
        private final ListMarker marker;
        private final List<DocumentElement> content;

        public ListItem(ListItem.Builder builder) {
            this.id = builder.id;
            this.marker = builder.marker;
            this.content = new ArrayList<>(builder.children);
        }

        @Override
        public <C> void accept(DocumentVisitor visitor, C context) {
            // ListItem wird durch ListElement-Visitor behandelt
        }

        @Override
        public String id() {
            return id;
        }

        public ListMarker getMarker() {
            return marker;
        }

        @Override
        public List<DocumentElement> getChildren() {
            return Collections.unmodifiableList(content);
        }


        public static class Builder implements ContainerElement.Builder {

            private String id;
            private ListMarker marker;
            private List<DocumentElement> children = new ArrayList<>();

            public Builder(String id) {
                this.id = id;
                this.marker = ListMarker.NONE;
            }

            public Builder setMarker(ListMarker marker) {
                this.marker = marker;
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

            public ListItem build() {
                return new ListItem(this);
            }
        }
    }
}