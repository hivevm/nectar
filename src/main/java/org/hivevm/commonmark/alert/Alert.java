// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause
package org.hivevm.commonmark.alert;

import java.util.Objects;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public enum Alert {

    NOTE,
    SUCCESS,
    WARNING,
    ERROR;

    private static final Pattern ALERT_LINE_PATTERN = Pattern.compile("^(!(!?)(\\w+)\\s+).*");

    static Matcher matcher(CharSequence text) {
        Objects.requireNonNull(text, "text");
        return ALERT_LINE_PATTERN.matcher(text);
    }

    static Alert of(String text) {
        if (text == null || text.isBlank()) {
            return NOTE;
        }

        String trimmed = text.trim();
        char firstChar = Character.toLowerCase(trimmed.charAt(0));

        return switch (firstChar) {
            case 'e' -> ERROR;   // Red, Error
            case 'w' -> WARNING; // Yellow, Warning
            case 's' -> SUCCESS; // Green, Success
            case 'n' -> NOTE;    // Blue, Info
            default -> NOTE;
        };
    }
}
