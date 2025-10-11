// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

/** The container defines a generic interface for all template items. */
public interface PageContainer {

  void addItem(PageRenderer.PageRenderable item);
}
