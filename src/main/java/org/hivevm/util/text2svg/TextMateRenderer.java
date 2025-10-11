// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util.text2svg;

import org.eclipse.tm4e.core.internal.grammar.ScopeStack;
import org.eclipse.tm4e.core.internal.theme.Theme;
import org.eclipse.tm4e.core.internal.theme.raw.RawThemeReader;
import org.eclipse.tm4e.core.registry.IGrammarSource;
import org.eclipse.tm4e.core.registry.IRegistryOptions;
import org.eclipse.tm4e.core.registry.IThemeSource;
import org.eclipse.tm4e.core.registry.Registry;
import org.hivevm.document.CodeBlock;
import org.hivevm.document.TextSpan;
import org.hivevm.document.TextStyle;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Objects;

public class TextMateRenderer {

    private static final String THEME = "/themes/github-light.json";

    private static final String GRAMMAR_CPP = "/grammars/cpp.json";
    private static final String GRAMMAR_INI = "/grammars/ini.json";
    private static final String GRAMMAR_XML = "/grammars/xml.json";
    private static final String GRAMMAR_JSON = "/grammars/json.json";
    private static final String GRAMMAR_JAVA = "/grammars/java.json";
    private static final String GRAMMAR_RUST = "/grammars/rust.json";
    private static final String GRAMMAR_YAML = "/grammars/yaml.json";

    private static Theme theme;
    private static Registry registry;

    private static Theme getTheme() throws Exception {
        if (theme == null) {
            var regOpt = new IRegistryOptions() {
            };
            theme = Theme.createFromRawTheme(
                    RawThemeReader.readTheme(IThemeSource.fromResource(TextMateRenderer.class, THEME)),
                    regOpt.getColorMap());
        }
        return theme;
    }

    private static void generate(String type, String text, List<CodeBlock.Line> rows) throws Exception {
        if (registry == null) {
            registry = new Registry();
            registry.setTheme(IThemeSource.fromResource(TextMateRenderer.class, THEME));
            registry.addGrammar(IGrammarSource.fromResource(TextMateRenderer.class, GRAMMAR_CPP));
            registry.addGrammar(IGrammarSource.fromResource(TextMateRenderer.class, GRAMMAR_INI));
            registry.addGrammar(IGrammarSource.fromResource(TextMateRenderer.class, GRAMMAR_XML));
            registry.addGrammar(IGrammarSource.fromResource(TextMateRenderer.class, GRAMMAR_JSON));
            registry.addGrammar(IGrammarSource.fromResource(TextMateRenderer.class, GRAMMAR_JAVA));
            registry.addGrammar(IGrammarSource.fromResource(TextMateRenderer.class, GRAMMAR_RUST));
            registry.addGrammar(IGrammarSource.fromResource(TextMateRenderer.class, GRAMMAR_YAML));
        }

        var theme = getTheme();

        String color = null;
        var spans = new ArrayList<TextSpan>();

        var grammar = "xml".equalsIgnoreCase(type)
                ? registry.grammarForScopeName("text." + type)
                : registry.grammarForScopeName("source." + type);
        var lineTokens = grammar.tokenizeLine(text);
        for (var token : lineTokens.getTokens()) {
            var fg = token.getScopes().reversed().stream()
                    .map(ScopeStack::from)
                    .map(theme::match)
                    .map(s -> s.foregroundId)
                    .map(i -> theme.getColorMap().get(i))
                    .findFirst().get();

            if (color == null)
                spans.add(new TextSpan.Builder().style(new TextStyle.Builder().color(fg).build()).build());
            else if (!Objects.equals(color, fg)) {
                spans.add(new TextSpan.Builder().end().build());
                spans.add(new TextSpan.Builder().style(new TextStyle.Builder().color(fg).build()).build());
            }
            color = fg;

            var image = text.substring(token.getStartIndex(), token.getEndIndex());
            spans.add(new TextSpan.Builder(image).scope(String.join(",", token.getScopes())).build());
        }
        spans.add(new TextSpan.Builder().end().build());
        rows.add(new CodeBlock.Line(spans));
    }

    public static List<CodeBlock.Line> render(String text, String type) throws Exception {
        var lines = new ArrayList<CodeBlock.Line>();
        TextMateRenderer.generate(type, text, lines);
        return lines;
    }

    public static org.hivevm.document.Theme getTheme(String name) throws Exception {
        var theme = getTheme();
        var root = theme.getRoot();
        return new org.hivevm.document.Theme(Map.of());
    }
}
