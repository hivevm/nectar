// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.grammar;

public interface ExpressionVisitor<R, C> {

    R visit(Choice elem, C context);

    R visit(Literal elem, C context);

    R visit(Repetition elem, C context);

    R visit(RuleReference elem, C context);

    R visit(Sequence elem, C context);

    R visit(SpecialSequence elem, C context);
}
