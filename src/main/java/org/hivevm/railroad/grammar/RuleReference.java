// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.grammar;

public record RuleReference(String ruleName) implements Expression {

    public <R, C> R accept(ExpressionVisitor<R, C> visitor, C context) {
        return visitor.visit(this, context);
    }
}
