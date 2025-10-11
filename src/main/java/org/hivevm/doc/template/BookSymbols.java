// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.net.URL;
import java.util.HashMap;
import java.util.Map;
import java.util.function.BiConsumer;
import java.util.function.Function;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Manages {@link BookSymbols}'s
 */
public class BookSymbols {

    private static final Pattern PATTERN     = Pattern.compile("^([^\\s]+)\\s([^\\s]+)$");
    private static final Pattern PLACEHOLDER = Pattern.compile("(@([^@]+)@)");

    private final Map<String, String>    FONTS   = new HashMap<>();
    private final Map<String, Character> SYMBOLS = new HashMap<>();

    /**
     * Avoid to create an instance of {@link BookSymbols}
     */
    protected BookSymbols() {
    }

    /**
     * Register the code-points for the font.
     *
     * @param name
     * @param codepoints
     */
    final void registerSymbols(String name, URL codepoints) {
        try (BufferedReader reader = new BufferedReader(
                new InputStreamReader(codepoints.openStream()))) {
            reader.lines().forEach(l -> {
                Matcher matcher = BookSymbols.PATTERN.matcher(l);
                if (matcher.find()) {
                    String symbol = matcher.group(1);
                    String value = matcher.group(2);
                    this.FONTS.put(symbol, name);
                    this.SYMBOLS.put(symbol, (char) Integer.parseInt(value, 16));
                }
            });
        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    /**
     * Renders each material symbol of the text using the provided {@link Function}.
     *
     * @param text
     * @param consumer
     */
    public final void forEach(String text, BiConsumer<String, String> consumer) {
        int offset = 0;

        Matcher matcher = BookSymbols.PLACEHOLDER.matcher(text);
        while (matcher.find()) {
            consumer.accept(text.substring(offset, matcher.start(1)), null);
            offset = matcher.end(1);

            String symbolName = matcher.group(2);
            if (this.SYMBOLS.get(symbolName) != null) {
                String fontName = this.FONTS.get(symbolName);
                Character symbol = this.SYMBOLS.get(symbolName);
                consumer.accept(symbol.toString(), fontName);
            } else {
                consumer.accept("@" + symbolName + "@", null);
            }
        }

        consumer.accept(text.substring(offset), null);
    }
}
