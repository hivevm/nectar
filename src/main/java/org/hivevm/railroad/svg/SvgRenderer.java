// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.svg;

import org.hivevm.railroad.Font;
import org.hivevm.railroad.Insets;
import org.hivevm.railroad.diagram.Railroad;

public class SvgRenderer implements Railroad.Visitor<Void, SvgContext> {

    @Override
    public Void visit(Railroad.Break request, SvgContext context) {
        throw new IllegalStateException("This element must not be nested and should have been processed before entering generation.");
    }

    @Override
    public Void visit(Railroad.Choice request, SvgContext context) {
        var layoutData = context.getLayoutData(request);
        int y1 = context.yOffset() + layoutData.connectorOffset();
        int x1 = context.xOffset() + 10;
        int x2 = context.xOffset() + layoutData.width() - 10;
        int xOffset2 = context.xOffset() + 20;
        int y2;
        int yOffset2 = context.yOffset();
        for (int i = 0; i < request.rrElements().size(); i++) {
            var rrElement = request.rrElements().get(i);
            var layoutInfo2 = context.getLayoutData(rrElement);
            int width = layoutInfo2.width();
            int height = layoutInfo2.height();
            y2 = yOffset2 + layoutInfo2.connectorOffset();
            if (i == 0) {
                // Line to first element
                context.content().addLineConnector(x1 - 10, y1, x1 + 10, y1);
            } else {
                if (i == request.rrElements().size() - 1) {
                    // Curve and vertical down
                    context.content().addPathConnector(x1 - 5, y1, "q5 0 5 5", x1, y1 + 5);
                    context.content().addLineConnector(x1, y1 + 5, x1, y2 - 5);
                }
                // Curve and horizontal line to element
                context.content().addPathConnector(x1, y2 - 5, "q0 5 5 5", x1 + 5, y2);
                context.content().addLineConnector(x1 + 5, y2, xOffset2, y2);
            }
            rrElement.accept(this, context.create(xOffset2, yOffset2));
            if (i == 0) {
                // Line to first element
                context.content().addLineConnector(xOffset2 + width, y2, x2 + 10, y2);
            } else {
                // Horizontal line to element and curve
                context.content().addLineConnector(x2 - 5, y2, xOffset2 + width, y2);
                context.content().addPathConnector(x2 - 5, y2, "q5 0 5-5", x2, y2 - 5);
                if (i == request.rrElements().size() - 1) {
                    // Vertical up and curve
                    context.content().addLineConnector(x2, y2 - 5, x2, y1 + 5);
                    context.content().addPathConnector(x2, y1 + 5, "q0-5 5-5", x2 + 5, y1);
                }
            }
            yOffset2 += height + 5;
        }
        return null;
    }

    @Override
    public Void visit(Railroad.Line request, SvgContext context) {
        return null;
    }

    @Override
    public Void visit(Railroad.Loop request, SvgContext context) {
        var layoutInfo1 = context.getLayoutData(request.rrElement());
        int width1 = layoutInfo1.width();
        int maxWidth = width1;
        int yOffset2 = context.yOffset();
        var layoutData = context.getLayoutData(request);
        int connectorOffset = layoutData.connectorOffset();
        int y1 = context.yOffset();
        int loopOffset = 0;
        int loopWidth = 0;
        if (request.loopElement() != null) {
            var layoutInfo2 = context.getLayoutData(request.loopElement());
            loopWidth = layoutInfo2.width();
            maxWidth = Math.max(maxWidth, loopWidth);
            loopOffset = context.xOffset() + 20 + (maxWidth - loopWidth) / 2;
            yOffset2 += 5 + layoutInfo2.height();
            y1 += layoutInfo2.connectorOffset();
        } else {
            yOffset2 += 15;
            y1 += 5;
        }
        int x1 = context.xOffset() + 10;
        int x2 = context.xOffset() + 20 + maxWidth + 10 + layoutData.cardinalitiesWidth();
        int y2 = context.yOffset() + connectorOffset;
        context.content().addLineConnector(x1 - 10, y2, x1 + 10 + (maxWidth - width1) / 2, y2);
        int loopPathStartX = x1 + 5;
        context.content().addPathConnector(x1 + 5, y2, "q-5 0-5-5", x1, y2 - 5);
        context.content().addLineConnector(x1, y2 - 5, x1, y1 + 5);
        context.content().addPathConnector(x1, y1 + 5, "q0-5 5-5", x1 + 5, y1);
        if (request.loopElement() != null) {
            context.content().addLineConnector(x1 + 5, y1, loopOffset, y1);
            request.loopElement().accept(this, context.create(loopOffset, context.yOffset()));
            loopPathStartX = loopOffset + loopWidth;
        }
        context.content().addLineConnector(loopPathStartX, y1, x2 - 5, y1);
        context.content().addPathConnector(x2 - 5, y1, "q5 0 5 5", x2, y1 + 5);
        context.content().addLineConnector(x2, y1 + 5, x2, y2 - 5);
        context.content().addPathConnector(x2, y2 - 5, "q0 5-5 5", x2 - 5, y2);
        if (layoutData.cardinalitiesText() != null) {
            String cssClass = context.content().getDefinedCSSClass(SvgDiagram.CSS_LOOP_CARDINALITIES_TEXT_CLASS);
            if (cssClass == null) {
                Font loopFont = context.layout().getLoopFont();
                String loopTextColor = Svg.toString(context.layout().getLoopTextColor());
                cssClass = context.content().setCSSClass(SvgDiagram.CSS_LOOP_CARDINALITIES_TEXT_CLASS, "fill:" + loopTextColor + ";" + Svg.toString(loopFont));
            }
            context.content().addElement("<text class=\"" + cssClass + "\" x=\"" + (x2 - layoutData.cardinalitiesWidth()) + "\" y=\"" + (y2 - context.yOffset() - 5) + "\">" + Svg.escapeXML(layoutData.cardinalitiesText()) + "</text>");
        }
        request.rrElement().accept(this, context.create(context.xOffset() + 20 + (maxWidth - width1) / 2, yOffset2));
        context.content().addLineConnector(x2 - layoutData.cardinalitiesWidth() - 10 - (maxWidth - width1) / 2, y2, context.xOffset() + layoutData.width(), y2);
        return null;
    }

    @Override
    public Void visit(Railroad.Sequence request, SvgContext context) {
        var layoutData = context.getLayoutData(request);
        int connectorOffset = layoutData.connectorOffset();
        int widthOffset = 0;
        for (int i = 0; i < request.rrElements().size(); i++) {
            var rrElement = request.rrElements().get(i);
            var layoutInfo2 = context.getLayoutData(rrElement);
            int width2 = layoutInfo2.width();
            int connectorOffset2 = layoutInfo2.connectorOffset();
            int xOffset2 = widthOffset + context.xOffset();
            int yOffset2 = context.yOffset() + connectorOffset - connectorOffset2;
            if (i > 0) {
                context.content().addLineConnector(xOffset2 - 10, context.yOffset() + connectorOffset, xOffset2, context.yOffset() + connectorOffset);
            }
            rrElement.accept(this, context.create(xOffset2, yOffset2));
            widthOffset += 10;
            widthOffset += width2;
        }
        return null;
    }

    @Override
    public Void visit(Railroad.ShapeElement request, SvgContext context) {
        var layoutData = context.getLayoutData(request);
        int width = layoutData.width();
        int height = layoutData.height();
        String connectorColor = Svg.toString(context.layout().getConnectorColor());
        switch (request.startEndShape()) {
            case EMPTY_CIRCLE: {
                double radius = (width - 1) / 2.0;
                String cssClass = context.content().setCSSClass(request.isStart() ? SvgDiagram.CSS_CONNECTOR_START_CLASS : SvgDiagram.CSS_CONNECTOR_END_CLASS, "fill:none;stroke:" + connectorColor + ";");
                context.content().addElement("<ellipse class=\"" + cssClass + "\" cx=\"" + (context.xOffset() + 0.5 + radius) + "\" cy=\"" + context.yOffset() + "\" rx=\"" + radius + "\" ry=\"" + radius + "\"/>");
                break;
            }
            case FILLED_CIRCLE: {
                double radius = (width - 1) / 2.0;
                String cssClass = context.content().setCSSClass(request.isStart() ? SvgDiagram.CSS_CONNECTOR_START_CLASS : SvgDiagram.CSS_CONNECTOR_END_CLASS, "fill:" + connectorColor + ";stroke:" + connectorColor + ";");
                context.content().addElement("<ellipse class=\"" + cssClass + "\" cx=\"" + (context.xOffset() + 0.5 + radius) + "\" cy=\"" + context.yOffset() + "\" rx=\"" + radius + "\" ry=\"" + radius + "\"/>");
                break;
            }
            case DOUBLE_VERTICAL_LINE: {
                double radius = height / 2.0;
                String cssClass = context.content().setCSSClass(request.isStart() ? SvgDiagram.CSS_CONNECTOR_START_CLASS : SvgDiagram.CSS_CONNECTOR_END_CLASS, "stroke:" + connectorColor + ";");
                double x = context.xOffset();
                if (request.isStart())
                    x++;

                context.content().addElement("<line class=\"" + cssClass + "\" x1=\"" + x + "\" x2=\"" + x + "\" y1=\"" + (context.yOffset() - radius) + "\" y2=\"" + (context.yOffset() + radius) + "\"/>");
                context.content().addElement("<line class=\"" + cssClass + "\" x1=\"" + (x + width - 1) + "\" x2=\"" + (x + width - 1) + "\" y1=\"" + (context.yOffset() - radius) + "\" y2=\"" + (context.yOffset() + radius) + "\"/>");
                break;
            }
            case VERTICAL_LINE: {
                double radius = height / 2.0;
                String cssClass = context.content().setCSSClass(request.isStart() ? SvgDiagram.CSS_CONNECTOR_START_CLASS : SvgDiagram.CSS_CONNECTOR_END_CLASS, "stroke:" + connectorColor + ";");
                double x = context.xOffset();
                if (request.isStart()) {
                    x++;
                }
                context.content().addElement("<line class=\"" + cssClass + "\" x1=\"" + x + "\" x2=\"" + x + "\" y1=\"" + (context.yOffset() - radius) + "\" y2=\"" + (context.yOffset() + radius) + "\"/>");
                break;
            }
            case EMPTY_ARROW: {
                double radius = height / 2.0;
                String cssClass = context.content().setCSSClass(request.isStart() ? SvgDiagram.CSS_CONNECTOR_START_CLASS : SvgDiagram.CSS_CONNECTOR_END_CLASS, "fill:none;stroke:" + connectorColor + ";");
                context.content().addElement("<polygon class=\"" + cssClass + "\" points=\"" + context.xOffset() + "," + (context.yOffset() - radius) + " " + context.xOffset() + "," + (context.yOffset() + radius) + " " + (context.xOffset() + width) + "," + context.yOffset() + "\"/>");
                break;
            }
            case FILLED_ARROW: {
                double radius = height / 2.0;
                String cssClass = context.content().setCSSClass(request.isStart() ? SvgDiagram.CSS_CONNECTOR_START_CLASS : SvgDiagram.CSS_CONNECTOR_END_CLASS, "stroke:" + connectorColor + ";");
                context.content().addElement("<polygon class=\"" + cssClass + "\" points=\"" + context.xOffset() + "," + (context.yOffset() - radius) + " " + context.xOffset() + "," + (context.yOffset() + radius) + " " + (context.xOffset() + width) + "," + context.yOffset() + "\"/>");
                break;
            }
        }
        return null;
    }

    @Override
    public Void visit(Railroad.Text request, SvgContext context) {
        var layoutData = context.getLayoutData(request);
        int width = layoutData.width();
        int height = layoutData.height();
        if (request.link() != null) {
            context.content().addElement("<a xlink:href=\"" + Svg.escapeXML(request.link())/* + "\" xlink:title=\"" + Svg.escapeXML(text)*/ + "\">");
        }
        Insets insets;
        Font font;
        String cssClass;
        String cssTextClass;
        SvgLayout.BoxShape shape;
        switch (request.type()) {
            case RULE:
                insets = context.layout().getRuleInsets();
                font = context.layout().getRuleFont();
                cssClass = context.content().getDefinedCSSClass(SvgDiagram.CSS_RULE_CLASS);
                cssTextClass = context.content().getDefinedCSSClass(SvgDiagram.CSS_RULE_TEXT_CLASS);
                if (cssClass == null) {
                    String ruleBorderColor = Svg.toString(context.layout().getRuleBorderColor());
                    String ruleFillColor = Svg.toString(context.layout().getRuleFillColor());
                    Font ruleFont = context.layout().getRuleFont();
                    String ruleTextColor = Svg.toString(context.layout().getRuleTextColor());
                    cssClass = context.content().setCSSClass(SvgDiagram.CSS_RULE_CLASS, "fill:" + ruleFillColor + ";stroke:" + ruleBorderColor + ";");
                    cssTextClass = context.content().setCSSClass(SvgDiagram.CSS_RULE_TEXT_CLASS, "fill:" + ruleTextColor + ";" + Svg.toString(ruleFont));
                }
                shape = context.layout().getRuleShape();
                break;
            case LITERAL:
                insets = context.layout().getLiteralInsets();
                font = context.layout().getLiteralFont();
                cssClass = context.content().getDefinedCSSClass(SvgDiagram.CSS_LITERAL_CLASS);
                cssTextClass = context.content().getDefinedCSSClass(SvgDiagram.CSS_LITERAL_TEXT_CLASS);
                if (cssClass == null) {
                    String literalBorderColor = Svg.toString(context.layout().getLiteralBorderColor());
                    String literalFillColor = Svg.toString(context.layout().getLiteralFillColor());
                    Font literalFont = context.layout().getLiteralFont();
                    String literalTextColor = Svg.toString(context.layout().getLiteralTextColor());
                    cssClass = context.content().setCSSClass(SvgDiagram.CSS_LITERAL_CLASS, "fill:" + literalFillColor + ";stroke:" + literalBorderColor + ";");
                    cssTextClass = context.content().setCSSClass(SvgDiagram.CSS_LITERAL_TEXT_CLASS, "fill:" + literalTextColor + ";" + Svg.toString(literalFont));
                }
                shape = context.layout().getLiteralShape();
                break;
            case SPECIAL_SEQUENCE:
                insets = context.layout().getSpecialSequenceInsets();
                font = context.layout().getSpecialSequenceFont();
                cssClass = context.content().getDefinedCSSClass(SvgDiagram.CSS_SPECIAL_SEQUENCE_CLASS);
                cssTextClass = context.content().getDefinedCSSClass(SvgDiagram.CSS_SPECIAL_SEQUENCE_TEXT_CLASS);
                if (cssClass == null) {
                    String specialSequenceBorderColor = Svg.toString(context.layout().getSpecialSequenceBorderColor());
                    String specialSequenceFillColor = Svg.toString(context.layout().getSpecialSequenceFillColor());
                    Font specialSequenceFont = context.layout().getSpecialSequenceFont();
                    String specialSequenceTextColor = Svg.toString(context.layout().getSpecialSequenceTextColor());
                    cssClass = context.content().setCSSClass(SvgDiagram.CSS_SPECIAL_SEQUENCE_CLASS, "fill:" + specialSequenceFillColor + ";stroke:" + specialSequenceBorderColor + ";");
                    cssTextClass = context.content().setCSSClass(SvgDiagram.CSS_SPECIAL_SEQUENCE_TEXT_CLASS, "fill:" + specialSequenceTextColor + ";" + Svg.toString(specialSequenceFont));
                }
                shape = context.layout().getSpecialSequenceShape();
                break;
            default:
                throw new IllegalStateException("Unknown type: " + request.type());
        }
        switch (shape) {
            case RECTANGLE:
                context.content().addElement("<rect class=\"" + cssClass + "\" x=\"" + context.xOffset() + "\" y=\"" + context.yOffset() + "\" width=\"" + width + "\" height=\"" + height + "\"/>");
                break;
            case ROUNDED_RECTANGLE:
                // Connector may be in rounded area if there are huge margins at top, but this is an unrealistic case so we don't add lines to complete the connector.
                int rx = (insets.left() + insets.right() + insets.top() + insets.bottom()) / 4;
                context.content().addElement("<rect class=\"" + cssClass + "\" x=\"" + context.xOffset() + "\" y=\"" + context.yOffset() + "\" width=\"" + width + "\" height=\"" + height + "\" rx=\"" + rx + "\"/>");
                break;
            case HEXAGON:
                // We don't calculate the exact length of the connector: it goes behind the shape.
                // We should calculate if we want to support transparent shapes.
                int connectorOffset = layoutData.connectorOffset();
                context.content().addLineConnector(context.xOffset(), context.yOffset() + connectorOffset, context.xOffset() + insets.left(), context.yOffset() + connectorOffset);
                context.content().addElement("<polygon class=\"" + cssClass + "\" points=\"" + context.xOffset() + " " + (context.yOffset() + height / 2) + " " + (context.xOffset() + insets.left()) + " " + context.yOffset() + " " + (context.xOffset() + width - insets.right()) + " " + context.yOffset() + " " + (context.xOffset() + width) + " " + (context.yOffset() + height / 2) + " " + (context.xOffset() + width - insets.right()) + " " + (context.yOffset() + height) + " " + (context.xOffset() + insets.left()) + " " + (context.yOffset() + height) + "\"/>");
                context.content().addLineConnector(context.xOffset() + width, context.yOffset() + connectorOffset, context.xOffset() + width - insets.right(), context.yOffset() + connectorOffset);
                break;
        }
        int textXOffset = context.xOffset() + insets.left();
        int textYOffset = context.yOffset() + insets.top() + font.getFontHeight(request.text()) - layoutData.fontYOffset();
        context.content().addElement("<text class=\"" + cssTextClass + "\" x=\"" + textXOffset + "\" y=\"" + textYOffset + "\">" + Svg.escapeXML(request.text()) + "</text>");
        if (request.link() != null) {
            context.content().addElement("</a>");
        }
        return null;
    }

    public static void render(Railroad elem, int xOffset, int yOffset, SvgLayout generator, SvgDiagram.SvgContent content) {
        var renderer = new SvgRenderer();
        var context = new SvgContext(xOffset, yOffset, generator, content);
        elem.accept(renderer, context);
    }
}
