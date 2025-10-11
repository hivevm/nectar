// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

import org.hivevm.util.text2svg.GraphvizRenderer;
import org.hivevm.util.text2svg.RailroadRenderer;
import org.hivevm.util.text2svg.TextMateRenderer;

import java.io.IOException;
import java.util.Base64;
import java.util.List;
import java.util.stream.Stream;

public class CodeBlock implements DocumentElement {

    private final String id;
    private final String language;
    private final List<Line> lines;
    private final boolean showLineNumbers;

    private CodeBlock(Builder builder) {
        this(builder, Stream.of(builder.code.split("\\n")).map(Line::new).toList());
    }

    private CodeBlock(Builder builder, List<Line> lines) {
        this.id = builder.id;
        this.language = builder.language;
        this.lines = lines;
        this.showLineNumbers = builder.showLineNumbers;
    }

    @Override
    public String id() {
        return id;
    }

    public List<Line> getLines() {
        return lines;
    }

    public String getLanguage() {
        return language;
    }

    public boolean isShowLineNumbers() {
        return showLineNumbers;
    }

    @Override
    public <C> void accept(DocumentVisitor visitor, C context) {
        visitor.visit(this, context);
    }

    public record Line(List<TextSpan> span) {

        private Line(String text) {
            this(List.of(new TextSpan.Builder(text).build()));
        }
    }

    public static class Builder {

        private String id;
        private String code;
        private String language = "";
        private boolean showLineNumbers = false;

        public Builder(String id, String code) {
            this.id = id;
            this.code = code;
        }

        public Builder language(String language) {
            this.language = language;
            return this;
        }

        public Builder showLineNumbers() {
            this.showLineNumbers = true;
            return this;
        }

        public DocumentElement build() {
            var lang = language != null ? language : "";
            return switch (lang) {
                case "uml", "ebnf", "regex", "jsongraph" -> {
                    try {
                        var type = "jsongraph".equalsIgnoreCase(lang) ? "json" : lang;
                        var bytes = GraphvizRenderer.render(code, type);
                        var base64 = Base64.getEncoder().encodeToString(bytes);
                        yield new Image(id, "data:image/svg+xml;base64," + base64);
                    } catch (IOException e) {
                    }
                    yield new CodeBlock(this);
                }
                case "dot" -> {
                    try {
                        var bytes = GraphvizRenderer.render(code);
                        var base64 = Base64.getEncoder().encodeToString(bytes);
                        yield new Image(id, "data:image/svg+xml;base64," + base64);
                    } catch (IOException e) {
                    }
                    yield new CodeBlock(this);
                }
                case "railroad" -> {
                    try {
                        var bytes = RailroadRenderer.render(code);
                        var base64 = Base64.getEncoder().encodeToString(bytes);
                        yield new Image(id, "data:image/svg+xml;base64," + base64);
                    } catch (IOException e) {
                        e.printStackTrace();
                    }
                    yield new CodeBlock(this);
                }
                case "ini", "yaml", "xml", "json", "java", "cpp", "rust" -> {
                    try {
                        var rows = TextMateRenderer.render(code, lang);
                        yield new CodeBlock(this, rows);
                    } catch (Exception e) {
                        e.printStackTrace();
                    }
                    yield new CodeBlock(this);
                }
                default -> new CodeBlock(this);
            };
        }
    }
}