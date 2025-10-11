// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

import java.net.URI;
import java.util.ArrayList;
import java.util.List;

/** The {@link TemplateFont} class. */
public class TemplateFont {

  private final String name;
  private final List<Metric> metrics = new ArrayList<>();


  /** Constructs an instance of {@link TemplateFont}. */
  public TemplateFont(String name) {
    this.name = name;
  }

  /** Gets the font name. */
  public final String getName() {
    return this.name;
  }

  /** Gets the collection of metrics. */
  public final Iterable<Metric> getMetrics() {
    return this.metrics;
  }

  /** Sets the {@link URI} to the font. */
  public final void addMetric(URI uri, boolean bold, boolean italic) {
    this.metrics.add(new Metric(uri, bold, italic));
  }

  /** The class defines the metric of a font */
  public record Metric(URI uri, boolean bold, boolean italic) {

  }
}
