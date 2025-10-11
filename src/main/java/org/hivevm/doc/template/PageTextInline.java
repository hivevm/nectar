// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

/** The {@link PageTextInline} class. */
public class PageTextInline {

  public String text;
  public String color;
  public String fontSize;
  public String fontStyle;
  public String fontWeight;
  public String fontFamily;
  public String textAlign;
  public String lineHeight;

  public final void setText(String text) {
    this.text = text;
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
}