// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.diagram;

import org.hivevm.railroad.grammar.Choice;
import org.hivevm.railroad.grammar.Expression;
import org.hivevm.railroad.grammar.ExpressionVisitor;
import org.hivevm.railroad.grammar.Literal;
import org.hivevm.railroad.grammar.Repetition;
import org.hivevm.railroad.grammar.RuleReference;
import org.hivevm.railroad.grammar.Sequence;
import org.hivevm.railroad.grammar.SpecialSequence;

class RailroadParser implements ExpressionVisitor<RailroadBuilder, RailroadBuilder> {

    @FunctionalInterface
    public interface RuleLinkProvider {

        String getLink(String ruleName);
    }

    @FunctionalInterface
    public interface SpecialSequenceElementProvider {

        Railroad getElement(String text);
    }

    private final RuleLinkProvider               ruleLinkProvider;
    private final String                         ruleConsideredAsLineBreak;
    private final SpecialSequenceElementProvider specialSequenceElementProvider;

    public RailroadParser() {
        this.ruleLinkProvider = ruleName -> "#" + ruleName;
        this.ruleConsideredAsLineBreak = null;
        this.specialSequenceElementProvider = null;
    }

    public RuleLinkProvider ruleLinkProvider() {
        return ruleLinkProvider;
    }

    public SpecialSequenceElementProvider specialSequenceElementProvider() {
        return specialSequenceElementProvider;
    }

    public String ruleConsideredAsLineBreak() {
        return ruleConsideredAsLineBreak;
    }

    @Override
    public final RailroadBuilder visit(Choice elem, RailroadBuilder context) {
        elem.expressions().forEach(r -> r.accept(this, context));
        return context.addChoice(elem.expressions().size());
    }

    @Override
    public final RailroadBuilder visit(Literal elem, RailroadBuilder context) {
        return context.addText(Railroad.Type.LITERAL, elem.text(), null);
    }

    @Override
    public final RailroadBuilder visit(Repetition elem, RailroadBuilder context) {
        context.add(elem.expression().accept(this, context).build());
        var maxRepetitionCount = (elem.maxRepetitionCount() == null ? null
            : elem.maxRepetitionCount() - 1);
        if (elem.minRepetitionCount() == 0) {
            if (elem.maxRepetitionCount() == null || elem.maxRepetitionCount() > 1)
                context.addLoop(0, maxRepetitionCount);
            return context.addLine().addChoice(2);
        }
        return context.addLoop(elem.minRepetitionCount() - 1, maxRepetitionCount);
    }

    @Override
    public final RailroadBuilder visit(RuleReference elem, RailroadBuilder context) {
        var lineBreak = ruleConsideredAsLineBreak();
        if (lineBreak != null && lineBreak.equals(elem.ruleName()))
            return context.addBreak();

        var linkProvider =
            ruleLinkProvider() == null ? null : ruleLinkProvider().getLink(elem.ruleName());
        return context.addText(Railroad.Type.RULE, elem.ruleName(), linkProvider);
    }

    @Override
    public final RailroadBuilder visit(Sequence elem, RailroadBuilder context) {
        var length = 0;
        for (int i = 0; i < elem.expressions().size(); i++) {
            Expression expression = elem.expressions().get(i);
            expression.accept(this, context);
            // Treat special case of: "a (',' a)*" and "a (a)*"
            if (i < elem.expressions().size() - 1 && elem.expressions()
                .get(i + 1) instanceof Repetition repetition) {
                Expression repetitionExpression = repetition.expression();
                if (repetitionExpression instanceof Sequence sequence) {
                    // Treat special case of: "expr (',' expr)*"
                    var subExpressions = sequence.expressions();
                    if (subExpressions.size() == 2 && subExpressions.get(0) instanceof Literal) {
                        if (expression.equals(subExpressions.get(1))) {
                            Integer maxRepetitionCount = repetition.maxRepetitionCount();
                            if (maxRepetitionCount == null || maxRepetitionCount > 1) {
                                subExpressions.getFirst().accept(this, context);
                                context.addLoop2(repetition.minRepetitionCount(),
                                    maxRepetitionCount);
                                i++;
                            }
                        }
                    }
                }
                else if (expression instanceof RuleReference ruleLink
                    && repetitionExpression instanceof RuleReference ruleReference
                    && ruleReference.ruleName().equals(ruleLink.ruleName())) {
                    // Treat special case of: a (a)*
                    Integer maxRepetitionCount = repetition.maxRepetitionCount();
                    if (maxRepetitionCount == null || maxRepetitionCount > 1) {
                        ruleLink.accept(this, context);
                        context.addLoop(repetition.minRepetitionCount(), maxRepetitionCount);
                        i++;
                    }
                }
            }
            length++;
        }
        return context.addSequence(length);
    }

    @Override
    public final RailroadBuilder visit(SpecialSequence elem, RailroadBuilder context) {
        var provider = specialSequenceElementProvider();
        Railroad element = provider != null ? provider.getElement(elem.text()) : null;
        return element != null
            ? context.add(element)
            : context.addText(Railroad.Text.Type.SPECIAL_SEQUENCE, elem.text(), null);
    }
}
