// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/** The {@link TemplateBuilder} class. */
class TemplateBuilder {

  private String font;
  private String width;
  private String height;


  private final BookSymbols symbols = new BookSymbols();
  private final Map<String, TemplateFont> fonts = new LinkedHashMap<>();

  private final List<PageStyle> styles = new ArrayList<>();
  private final Map<String, Page> pages = new LinkedHashMap<>();
  private final Map<String, PageSet> pageSets = new LinkedHashMap<>();

  /** Gets the default width. */
  public final String getWidth() {
    return this.width;
  }

  /** Gets the default height. */
  public final String getHeight() {
    return this.height;
  }

  public final void addFontSymbols(String name, java.net.URL codepoints) {
    this.symbols.registerSymbols(name, codepoints);
  }

  public final TemplateBuilder addStyle(PageStyle style) {
    this.styles.add(style);
    return this;
  }

  /** Get {@link Page} by name. */
  public final Page getTemplate(String name) {
    return this.pages.get(name);
  }

  public final TemplateBuilder setFont(String font) {
    this.font = font;
    return this;
  }

  /** Sets the default size. */
  public final TemplateBuilder setSize(String width, String height) {
    this.width = width;
    this.height = height;
    return this;
  }

  /** Add a named {@link Page}. */
  public final Page addPage(String name) {
    var page = new Page(name);
    this.pages.put(name, page);
    return page;
  }

  /** Add a named {@link Page}. */
  public final PageSet addPageSet(String name) {
    var pageSet = new PageSet(name);
    this.pageSets.put(name, pageSet);
    return pageSet;
  }

  /** Adds a font by name. */
  public final TemplateFont addFont(String name) {
    var font = new TemplateFont(name);
    this.fonts.put(name, font);
    return font;
  }

  /** Creates the {@link Template}. */
  public final Template build() {
    return new Template(font, width, height, symbols, fonts.values(), styles, pages.values(),
        pageSets.values());
  }
}
