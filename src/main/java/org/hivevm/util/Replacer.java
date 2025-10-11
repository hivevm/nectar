// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util;

import java.util.Map;
import java.util.function.Function;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * The {@link Replacer} class.
 */
public class Replacer {

  // Environment variables: {{ENVIRONMENT_VARIABLE;FORMATTER}}
  private static final String       PATTERN  = "\\{\\{([^};]+)(?:;([^}]+))?}}";
  private static final Pattern      TEMPLATE = Pattern.compile(Replacer.PATTERN, Pattern.CASE_INSENSITIVE);


  private final Map<String, String> properties;

  /**
   * Constructs an instance of {@link Replacer}.
   *
   * @param properties
   */
  public Replacer(Map<String, String> properties) {
    this.properties = properties;
  }

  /**
   * Get an environment variable by name.
   *
   * @param name
   * @param format
   */
  private final String getProperty(String name, String format) {
    if (this.properties.containsKey(name)) {
      return this.properties.get(name);
    }
    return "{{" + name + ((format == null) ? "" : ";" + format) + "}}";
  }

  /**
   * Replace all environment variables on the text
   *
   * @param text
   */
  public final String replaceAll(String text) {
    return Replacer.replaceAll(text, Replacer.TEMPLATE, m -> getProperty(m.group(1), m.group(2)));
  }

  /**
   * Uses the {@link Pattern} to replace parts of the text.
   *
   * @param text
   * @param pattern
   * @param function
   */
  public static String replaceAll(String text, Pattern pattern, Function<Matcher, String> function) {
    StringBuilder content = new StringBuilder();
    Matcher matcher = pattern.matcher(text);

    int offset = 0;
    while (matcher.find()) {
      String result = function.apply(matcher);
      if (result != null) {
        content.append(text, offset, matcher.start());
        content.append(result);
        offset = matcher.end();
      }
    }
    content.append(text.substring(offset));
    return content.toString();
  }
}
