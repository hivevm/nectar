// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.diagram;

import java.util.List;
import org.hivevm.railroad.grammar.Expression;

public interface Railroad {

    <R, C> R accept(Visitor<R, C> visitor, C context);

    enum Type {
        LITERAL,
        RULE,
        SPECIAL_SEQUENCE,
    }

    enum StartEndShape {
        EMPTY_CIRCLE,
        FILLED_CIRCLE,
        VERTICAL_LINE,
        DOUBLE_VERTICAL_LINE,
        EMPTY_ARROW,
        FILLED_ARROW,
    }

    record Break() implements Railroad {

        public <R, C> R accept(Visitor<R, C> visitor, C context) {
            return visitor.visit(this, context);
        }
    }

    record Choice(List<Railroad> rrElements) implements Railroad {

        public <R, C> R accept(Visitor<R, C> visitor, C context) {
            return visitor.visit(this, context);
        }
    }

    record Line() implements Railroad {

        public <R, C> R accept(Visitor<R, C> visitor, C context) {
            return visitor.visit(this, context);
        }
    }

    record Loop(Railroad rrElement, Railroad loopElement, int minRepetitionCount,
                Integer maxRepetitionCount) implements Railroad {

        public <R, C> R accept(Visitor<R, C> visitor, C context) {
            return visitor.visit(this, context);
        }

        public static Loop create(Railroad rrElement, Railroad loopElement, int minRepetitionCount, Integer maxRepetitionCount) {
            if (minRepetitionCount < 0) {
                throw new IllegalArgumentException("Minimum repetition must be positive!");
            }
            if (maxRepetitionCount != null && maxRepetitionCount < minRepetitionCount) {
                throw new IllegalArgumentException("Maximum repetition must not be smaller than minimum!");
            }
            return new Loop(rrElement, loopElement, minRepetitionCount, maxRepetitionCount);
        }
    }

    record Sequence(List<Railroad> rrElements) implements Railroad {

        public <R, C> R accept(Visitor<R, C> visitor, C context) {
            return visitor.visit(this, context);
        }
    }

    record ShapeElement(StartEndShape startEndShape,
                        boolean isStart) implements Railroad {

        public <R, C> R accept(Visitor<R, C> visitor, C context) {
            return visitor.visit(this, context);
        }
    }

    record Text(Type type, String text,
                String link) implements Railroad {


        public <R, C> R accept(Visitor<R, C> visitor, C context) {
            return visitor.visit(this, context);
        }
    }

    interface Visitor<R, C> {

        R visit(Break request, C context);

        R visit(Choice request, C context);

        R visit(Line request, C context);

        R visit(Loop request, C context);

        R visit(Sequence request, C context);

        R visit(ShapeElement request, C context);

        R visit(Text request, C context);
    }

    static Railroad toRailroad(Expression expr) {
        return expr.accept(new RailroadParser(), new RailroadBuilder()).build();
    }
}