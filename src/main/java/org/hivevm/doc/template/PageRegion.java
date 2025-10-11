// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;

/** The {@link PageRegion} class. */
public class PageRegion implements PageContainer {

  public enum Region {
    BEFORE,
    AFTER,
    START,
    END
  }

  private final Page page;
  private final Region region;

  private String extent;

  private final List<PageColumn> children = new ArrayList<>();

  /** Constructs an instance of {@link PageRegion}. */
  public PageRegion(Page page, Region region) {
    this.page = page;
    this.region = region;
  }

  public String getName() {
    return String.format("region-%s-%s", page.getName(), region.name());
  }

  public Region getRegion() {
    return region;
  }

  public String getExtent() {
    return extent;
  }

  public final void addItem(PageRenderer.PageRenderable item) {
    this.children.add((PageColumn) item);
  }

  final void setExtent(String extent) {
    this.extent = extent;
  }

  public boolean hasChildren() {
    return !children.isEmpty();
  }

  public final void forEachItem(Consumer<PageColumn> action) {
    children.forEach(action);
  }
}
