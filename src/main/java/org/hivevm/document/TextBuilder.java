// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

public interface TextBuilder<B extends TextBuilder<?>> {

    B addText(TextSpan textSpan);
}
