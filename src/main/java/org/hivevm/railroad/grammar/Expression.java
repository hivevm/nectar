// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.grammar;

public interface Expression {

    <R, C> R accept(ExpressionVisitor<R, C> visitor, C context);

    static GrammarBuilder builder() {
        return new GrammarBuilder();
    }
}
