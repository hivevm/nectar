// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.bnf;

import java.util.ArrayList;
import java.util.List;
import org.hivevm.railroad.grammar.Choice;
import org.hivevm.railroad.grammar.Expression;
import org.hivevm.railroad.grammar.ExpressionVisitor;
import org.hivevm.railroad.grammar.Grammar;
import org.hivevm.railroad.grammar.Literal;
import org.hivevm.railroad.grammar.Repetition;
import org.hivevm.railroad.grammar.Rule;
import org.hivevm.railroad.grammar.RuleReference;
import org.hivevm.railroad.grammar.Sequence;
import org.hivevm.railroad.grammar.SpecialSequence;

public class BNFWriter implements ExpressionVisitor<Void, Boolean> {

    public enum RuleDefinitionSign {
        EQUAL("="),
        COLON_EQUAL(":="),
        COLON_COLON_EQUAL("::=");

        final String VALUE;

        RuleDefinitionSign(String value) {
            this.VALUE = value;
        }
    }

    public enum LiteralDefinitionSign {
        QUOTE("'"),
        DOUBLE_QUOTE("\"");

        final String VALUE;

        LiteralDefinitionSign(String value) {
            this.VALUE = value;
        }
    }


    private final StringBuilder         builder;
    private final RuleDefinitionSign    ruleDefinitionSign;
    private final LiteralDefinitionSign literalDefinitionSign;

    private final boolean isCommaSeparator;
    private final boolean isUsingMultiplicationTokens;
    private final String  ruleConsideredAsLineBreak;

    public BNFWriter() {
        this.builder = new StringBuilder();
        this.ruleDefinitionSign = RuleDefinitionSign.EQUAL;
        this.literalDefinitionSign = LiteralDefinitionSign.QUOTE;
        this.isCommaSeparator = false;
        this.isUsingMultiplicationTokens = false;
        this.ruleConsideredAsLineBreak = null;
    }

    public final void append(String s) {
        builder.append(s);
    }

    public final RuleDefinitionSign ruleDefinitionSign() {
        return ruleDefinitionSign;
    }

    public final LiteralDefinitionSign literalDefinitionSign() {
        return literalDefinitionSign;
    }

    public final boolean isCommaSeparator() {
        return isCommaSeparator;
    }

    public final boolean isUsingMultiplicationTokens() {
        return isUsingMultiplicationTokens;
    }

    public final String ruleConsideredAsLineBreak() {
        return ruleConsideredAsLineBreak;
    }

    public static String toBNF(Expression expression) {
        var writer = new BNFWriter();
        expression.accept(writer, false);
        return writer.builder.toString();
    }

    public static String toBNF(Grammar grammar) {
        var writer = new BNFWriter();
        for (int i = 0; i < grammar.rules().size(); i++) {
            if (i > 0)
                writer.append("\n");
            toBNF(writer, grammar.rules().get(i));
        }
        return writer.builder.toString();
    }

    public static String toBNF(Rule rule) {
        var writer = new BNFWriter();
        toBNF(writer, rule);
        return writer.builder.toString();
    }

    private static void toBNF(BNFWriter writer, Rule rule) {
        writer.append(rule.name());
        writer.append(" ");
        writer.append(writer.ruleDefinitionSign().VALUE);
        writer.append(" ");
        rule.expression().accept(writer, false);
        writer.append(";");
    }

    @Override
    public Void visit(Choice elem, Boolean isNested) {
        List<Expression> expressionList = new ArrayList<>();
        boolean hasNoop = false;
        for (Expression expression : elem.expressions()) {
            if (expression instanceof Sequence && ((Sequence) expression).expressions().isEmpty())
                hasNoop = true;
            else
                expressionList.add(expression);
        }
        if (expressionList.isEmpty())
            append("( )");
        else if (hasNoop && expressionList.size() == 1) {
            if (!isUsingMultiplicationTokens())
                append("[ ");
            expressionList.getFirst().accept(this, isUsingMultiplicationTokens());
            if (!isUsingMultiplicationTokens())
                append(" ]");
        } else {
            if (hasNoop && !isUsingMultiplicationTokens())
                append("[ ");
            else if (hasNoop || isNested && expressionList.size() > 1)
                append("( ");
            int count = expressionList.size();
            for (int i = 0; i < count; i++) {
                if (i > 0)
                    append(" | ");
                expressionList.get(i).accept(this, false);
            }
            if (hasNoop && !isUsingMultiplicationTokens())
                append(" ]");
            else if (hasNoop || isNested && expressionList.size() > 1) {
                append(" )");
                if (hasNoop)
                    append("?");
            }
        }
        return null;
    }

    @Override
    public Void visit(Literal elem, Boolean isNested) {
        append(literalDefinitionSign().VALUE);
        append(elem.text());
        append(literalDefinitionSign().VALUE);
        return null;
    }

    @Override
    public Void visit(Repetition elem, Boolean isNested) {
        if (elem.maxRepetitionCount() == null) {
            if (elem.minRepetitionCount() > 0) {
                if (elem.minRepetitionCount() == 1 && isUsingMultiplicationTokens()) {
                    elem.expression().accept(this, true);
                    append("+");
                } else {
                    if (isNested)
                        append("( ");
                    if (elem.minRepetitionCount() > 1) {
                        append("" + elem.minRepetitionCount());
                        append(" * ");
                    }
                    elem.expression().accept(this, false);
                    if (isCommaSeparator())
                        append(" ,");
                    append(" ");
                    append("{ ");
                    elem.expression().accept(this, false);
                    append(" }");
                    if (isNested)
                        append(" )");
                }
            } else {
                if (isUsingMultiplicationTokens()) {
                    elem.expression().accept(this, true);
                    append("*");
                } else {
                    append("{ ");
                    elem.expression().accept(this, false);
                    append(" }");
                }
            }
        } else {
            if (elem.minRepetitionCount() == 0) {
                if (elem.maxRepetitionCount() == 1 && isUsingMultiplicationTokens()) {
                    elem.expression().accept(this, true);
                    append("?");
                } else {
                    if (elem.maxRepetitionCount() > 1) {
                        append("" + elem.maxRepetitionCount());
                        append(" * ");
                    }
                    append("[ ");
                    elem.expression().accept(this, false);
                    append(" ]");
                }
            } else {
                if (elem.minRepetitionCount() == elem.maxRepetitionCount()) {
                    append("" + elem.minRepetitionCount());
                    append(" * ");
                    elem.expression().accept(this, isNested);
                } else {
                    if (isNested)
                        append("( ");
                    append("" + elem.minRepetitionCount());
                    append(" * ");
                    elem.expression().accept(this, false);
                    if (isCommaSeparator())
                        append(" ,");
                    append(" ");
                    append("" + (elem.maxRepetitionCount() - elem.minRepetitionCount()));
                    append(" * ");
                    append("[ ");
                    elem.expression().accept(this, false);
                    append(" ]");
                    if (isNested)
                        append(" )");
                }
            }
        }
        return null;
    }

    @Override
    public Void visit(RuleReference elem, Boolean isNested) {
        append(elem.ruleName());
        if (elem.ruleName().equals(ruleConsideredAsLineBreak()))
            append("\n");
        return null;
    }

    @Override
    public Void visit(Sequence elem, Boolean isNested) {
        if (elem.expressions().isEmpty()) {
            append("( )");
            return null;
        }
        if (isNested && elem.expressions().size() > 1)
            append("( ");
        for (int i = 0; i < elem.expressions().size(); i++) {
            if (i > 0) {
                if (isCommaSeparator())
                    append(" ,");
                append(" ");
            }
            elem.expressions().get(i).accept(this, elem.expressions().size() == 1 && isNested || !isCommaSeparator());
        }
        if (isNested && elem.expressions().size() > 1)
            append(" )");
        return null;
    }

    @Override
    public Void visit(SpecialSequence elem, Boolean isNested) {
        append("(? ");
        append(elem.text());
        append(" ?)");
        return null;
    }
}
