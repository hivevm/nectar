// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.grammar;

import java.util.Arrays;
import java.util.Stack;

public class GrammarBuilder {

    private final Stack<Expression> stack = new Stack<>();

    public final Expression current() {
        return stack.isEmpty() ? null : stack.peek();
    }

    public final GrammarBuilder add(Expression expr) {
        stack.push(expr);
        return this;
    }

    public final GrammarBuilder choice(int count) {
        var array = new Expression[count];
        while (count > 0)
            array[--count] = stack.pop();
        return add(new Choice(Arrays.asList(array)));
    }

    public final GrammarBuilder literal(String text) {
        return add(new Literal(text));
    }

    public final GrammarBuilder repetition(int minRepetitionCount, Integer maxRepetitionCount) {
        var expr = stack.pop();
        return add(new Repetition(expr, minRepetitionCount, maxRepetitionCount));
    }

    public final GrammarBuilder ruleReference(String name) {
        return add(new RuleReference(name));
    }

    public final GrammarBuilder sequence(int count) {
        var array = new Expression[count];
        while (count > 0)
            array[--count] = stack.pop();
        return add(new Sequence(Arrays.asList(array)));
    }

    public final GrammarBuilder specialSequence(String text) {
        return add(new SpecialSequence(text));
    }

    public final Expression build() {
        return stack.pop();
    }
}
