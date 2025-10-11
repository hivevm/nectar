// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

import java.io.File;
import java.io.IOException;
import java.util.Collection;
import java.util.List;
import org.hivevm.util.DataUri;
import org.hivevm.util.xml.StAX;

/** Defines a Template definition for a collection of pages. */
public class Template {

  private final String font;
  private final String width;
  private final String height;

  private final Collection<TemplateFont> fonts;
  private final BookSymbols symbols;
  private final List<PageStyle> styles;

  private final Collection<Page> pages;
  private final Collection<PageSet> pageSets;

  public Template(String font, String width, String height, BookSymbols symbols,
      Collection<TemplateFont> fonts,
      List<PageStyle> styles, Collection<Page> pages, Collection<PageSet> pageSets) {
    this.font = font;
    this.width = width;
    this.height = height;
    this.symbols = symbols;
    this.fonts = fonts;
    this.styles = styles;
    this.pages = pages;
    this.pageSets = pageSets;
  }

  /** Gets the default font */
  public final String getFont() {
    return font;
  }

  /** Gets the default width. */
  public final String getWidth() {
    return width;
  }

  /** Gets the default height. */
  public final String getHeight() {
    return height;
  }

  /** Gets the {@link TemplateFont}'s. */
  public final Collection<TemplateFont> getFonts() {
    return fonts;
  }

  /** Gets the {@link BookSymbols}. */
  public final BookSymbols getSymbols() {
    return symbols;
  }

  /** Gets the {@link BookSymbols}. */
  public final PageStyle getStyle(String name) {
    return styles.stream().filter(s -> s.matches(name)).findFirst().orElse(null);
  }

  public final Iterable<Page> pages() {
    return pages;
  }

  public final Iterable<PageSet> pageSet() {
    return pageSets;
  }

  /** Parses the configuration from the file. */
  public static Template getDefault() throws IOException {
    var builder = new TemplateBuilder();
    var parser = new TemplateParser(builder, new File("."));
    try (var stream = DataUri.toInputStream(":default.ui.xml")) {
      StAX.parse(stream, parser);
    }
    return builder.build();
  }

  /** Parses the configuration from the file. */
  public static Template parse(String config, File workingDir) throws IOException {
    var builder = new TemplateBuilder();
    var parser = new TemplateParser(builder, workingDir);
    try (var stream = DataUri.toInputStream(":default.ui.xml")) {
      StAX.parse(stream, parser);
    }
    try (var stream = DataUri.toInputStream(config, workingDir)) {
      StAX.parse(stream, parser);
    }
    return builder.build();
  }
}
