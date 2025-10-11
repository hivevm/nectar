// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;

/** The {@link PageRow} class. */
public class PageRow extends PageRenderer.PageRenderable {

  public String color;
  public String fontSize;
  public String fontStyle;
  public String fontWeight;
  public String fontFamily;
  public String textAlign;
  public String lineHeight;

  public final List<PageTextInline> items = new ArrayList<>();

  public final void addText(String text) {
    PageTextInline inline = new PageTextInline();
    inline.setText(text);
    this.items.add(inline);
  }

  public final void setColor(String color) {
    this.color = color;
  }

  public final void setFontSize(String fontSize) {
    this.fontSize = fontSize;
  }

  public final void setFontStyle(String fontStyle) {
    this.fontStyle = fontStyle;
  }

  public final void setFontWeight(String fontWeight) {
    this.fontWeight = fontWeight;
  }

  public final void setFontFamily(String fontFamily) {
    this.fontFamily = fontFamily;
  }

  public final void setTextAlign(String textAlign) {
    this.textAlign = textAlign;
  }

  public final void setLineHeight(String lineHeight) {
    this.lineHeight = lineHeight;
  }


  public final void addTextItem(PageTextInline inline) {
    this.items.add(inline);
  }

  @Override
  public final <B> void accept(PageRenderer<B> visitor, B block) {
    visitor.visit(this, block);
  }

  public final void forEachInline(Consumer<PageTextInline> action) {
    items.forEach(action);
  }
}