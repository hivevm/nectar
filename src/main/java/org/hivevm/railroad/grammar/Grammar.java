// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.grammar;

import java.util.ArrayList;
import java.util.List;

public class Grammar {

    private final List<Rule> rules;

    public Grammar() {
        this.rules = new ArrayList<>();
    }

    public final Grammar addRule(String name, Expression expression, String originalExpressionText) {
        rules.add(new Rule(name, expression, originalExpressionText));
        return this;
    }

    public final List<Rule> rules() {
        return rules;
    }
}
