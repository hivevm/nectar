// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.diagram;


import org.hivevm.railroad.Font;
import org.hivevm.railroad.Insets;

class LayoutDataBuilder implements Railroad.Visitor<LayoutData, LayoutContext> {

    @Override
    public LayoutData visit(Railroad.Break request, LayoutContext context) {
        throw new IllegalStateException("This element must not be nested and should have been processed before entering generation.");
    }

    @Override
    public LayoutData visit(Railroad.Choice request, LayoutContext context) {
        int width = 0;
        int height = 0;
        int connectorOffset = 0;
        for (int i = 0; i < request.rrElements().size(); i++) {
            Railroad rrElement = request.rrElements().get(i);
            LayoutData layoutInfo = rrElement.accept(this, context);
            context.setLayoutData(rrElement, layoutInfo);
            if (i == 0) {
                connectorOffset = layoutInfo.connectorOffset();
            } else {
                height += 5;
            }
            height += layoutInfo.height();
            width = Math.max(width, layoutInfo.width());
        }
        width += 20 + 20;
        return new LayoutData(width, height, connectorOffset);
    }

    @Override
    public LayoutData visit(Railroad.Line request, LayoutContext context) {
        return new LayoutData(0, 10, 5);
    }

    @Override
    public LayoutData visit(Railroad.Loop request, LayoutContext context) {
        String cardinalitiesText = null;
        int cardinalitiesWidth = 0;
        int fontYOffset = 0;
        if (request.minRepetitionCount() > 0 || request.maxRepetitionCount() != null) {
            cardinalitiesText = request.minRepetitionCount() + ".." + (request.maxRepetitionCount() == null ? "N" : request.maxRepetitionCount());
            Font font = context.getLoopFont();
            fontYOffset = font.getLineMetrics(cardinalitiesText);
            cardinalitiesWidth = font.getFontWidth(cardinalitiesText) + 2;

        }
        LayoutData layoutInfo1 = request.rrElement().accept(this, context);
        context.setLayoutData(request.rrElement(), layoutInfo1);
        int width = layoutInfo1.width();
        int height = layoutInfo1.height();
        int connectorOffset = layoutInfo1.connectorOffset();
        if (request.loopElement() != null) {
            LayoutData layoutInfo2 = request.loopElement().accept(this, context);
            context.setLayoutData(request.loopElement(), layoutInfo2);
            width = Math.max(width, layoutInfo2.width());
            int height2 = layoutInfo2.height();
            height += 5 + height2;
            connectorOffset += 5 + height2;
        } else {
            height += 15;
            connectorOffset += 15;
        }
        width += 20 + 20 + cardinalitiesWidth;
        return new LayoutData(width, height, connectorOffset, fontYOffset, cardinalitiesText, cardinalitiesWidth);
    }

    @Override
    public LayoutData visit(Railroad.Sequence request, LayoutContext context) {
        int width = 0;
        int aboveConnector = 0;
        int belowConnector = 0;
        for (int i = 0; i < request.rrElements().size(); i++) {
            Railroad rrElement = request.rrElements().get(i);
            LayoutData layoutInfo = rrElement.accept(this, context);
            if (i > 0) {
                width += 10;
            }
            context.setLayoutData(rrElement, layoutInfo);
            width += layoutInfo.width();
            int height = layoutInfo.height();
            int connectorOffset = layoutInfo.connectorOffset();
            aboveConnector = Math.max(aboveConnector, connectorOffset);
            belowConnector = Math.max(belowConnector, height - connectorOffset);
        }
        return new LayoutData(width, aboveConnector + belowConnector, aboveConnector);
    }

    @Override
    public LayoutData visit(Railroad.ShapeElement request, LayoutContext context) {
        int width = 10;
        int height = 10;
        switch (request.startEndShape()) {
            case EMPTY_CIRCLE:
            case FILLED_CIRCLE:
                width = 10;
                height = 10;
                break;
            case DOUBLE_VERTICAL_LINE:
                width = 4;
                height = 10;
                break;
            case VERTICAL_LINE:
                width = 1;
                height = 10;
                break;
            case EMPTY_ARROW:
            case FILLED_ARROW:
                width = 8;
                height = 10;
                break;
        }
        return new LayoutData(width, height, 0);
    }

    @Override
    public LayoutData visit(Railroad.Text request, LayoutContext context) {
        Font font;
        Insets insets;
        switch (request.type()) {
            case RULE:
                insets = context.getRuleInsets();
                font = context.getRuleFont();
                break;
            case LITERAL:
                insets = context.getLiteralInsets();
                font = context.getLiteralFont();
                break;
            case SPECIAL_SEQUENCE:
                insets = context.getSpecialSequenceInsets();
                font = context.getSpecialSequenceFont();
                break;
            default:
                throw new IllegalStateException("Unknown type: " + request.type());
        }
        int fontYOffset = font.getLineMetrics(request.text());
        int width = font.getFontWidth(request.text());
        int height = font.getFontHeight(request.text());
        boolean isAlignedOnBaseLine = context.isRuleTextAlignedOnBaseLine();
        int connectorOffset = insets.top() + (isAlignedOnBaseLine ? height - fontYOffset : height / 2);
        width += insets.left() + insets.right();
        height += insets.top() + insets.bottom();
        return new LayoutData(width, height, connectorOffset, fontYOffset);
    }
}
