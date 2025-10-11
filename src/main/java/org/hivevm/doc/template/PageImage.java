// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

import java.net.URI;


/** The {@link PageImage} class. */
public class PageImage extends PageRenderer.PageRenderable {

  public final URI uri;

  /** Constructs an instance of {@link PageImage}. */
  public PageImage(URI uri) {
    this.uri = uri;
  }

  @Override
  public final <B> void accept(PageRenderer<B> visitor, B block) {
    visitor.visit(this, block);
  }
}