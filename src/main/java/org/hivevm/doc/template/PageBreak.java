// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;


/** The {@link PageBreak} class. */
public class PageBreak extends PageRenderer.PageRenderable {

  public final String size;
  public final String style;
  public final String color;

  /** Constructs an instance of {@link PageBreak}. */
  public PageBreak(String size, String style, String color) {
    this.size = size;
    this.style = style;
    this.color = color;
  }

  @Override
  public final <B> void accept(PageRenderer<B> visitor, B block) {
    visitor.visit(this, block);
  }
}