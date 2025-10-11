// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.svg;


import java.util.HashMap;
import java.util.Map;
import org.hivevm.railroad.Color;
import org.hivevm.railroad.Font;
import org.hivevm.railroad.Insets;
import org.hivevm.railroad.diagram.LayoutContext;
import org.hivevm.railroad.diagram.LayoutData;
import org.hivevm.railroad.diagram.Railroad;

public class SvgLayout implements LayoutContext {

    private Color    connectorColor              = new Color(34, 34, 34);
    private Font     loopFont                    = new Font("Verdana", 10);
    private Color    loopTextColor               = Color.BLACK;
    private Insets   ruleInsets                  = new Insets(5, 10, 5, 10);
    private Font     ruleFont                    = new Font("Verdana", 12);
    private Color    ruleTextColor               = Color.BLACK;
    private boolean  isRuleTextAlignedOnBaseLine = true;
    private BoxShape ruleShape                   = BoxShape.RECTANGLE;
    private Color    ruleBorderColor             = connectorColor;
    private Color    ruleFillColor               = new Color(211, 240, 255);
    private Insets   literalInsets               = new Insets(5, 10, 5, 10);
    private Font     literalFont                 = new Font("Verdana", 12);

    private Color    literalTextColor   = Color.BLACK;
    private BoxShape literalShape       = BoxShape.ROUNDED_RECTANGLE;
    private Color    literalBorderColor = connectorColor;
    private Color    literalFillColor   = new Color(144, 217, 255);

    private Insets               specialSequenceInsets      = new Insets(5, 10, 5, 10);
    private Font                 specialSequenceFont        = new Font("Verdana", 12);
    private Color                specialSequenceTextColor   = Color.BLACK;
    private BoxShape             specialSequenceShape       = BoxShape.HEXAGON;
    private Color                specialSequenceBorderColor = connectorColor;
    private Color                specialSequenceFillColor   = new Color(228, 244, 255);
    private LineContinuationType lineContinuationType       = LineContinuationType.NONE;
    private boolean              isLeftAligned              = true;
    private Railroad             startElement               = null;
    private Railroad             endElement                 = null;

    private final Map<Railroad, LayoutData> layoutData = new HashMap<>();

    public void setConnectorColor(Color connectorColor) {
        this.connectorColor = connectorColor;
    }

    public Color getConnectorColor() {
        return connectorColor;
    }

    public void setLoopFont(Font loopFont) {
        this.loopFont = loopFont;
    }

    public Font getLoopFont() {
        return loopFont;
    }

    public void setLoopTextColor(Color loopTextColor) {
        this.loopTextColor = loopTextColor;
    }

    public Color getLoopTextColor() {
        return loopTextColor;
    }

    public enum BoxShape {
        RECTANGLE, ROUNDED_RECTANGLE, HEXAGON
    }

    public void setRuleInsets(Insets ruleInsets) {
        this.ruleInsets = ruleInsets;
    }

    public Insets getRuleInsets() {
        return ruleInsets;
    }


    public void setRuleFont(Font ruleFont) {
        this.ruleFont = ruleFont;
    }

    public Font getRuleFont() {
        return ruleFont;
    }


    public void setRuleTextColor(Color ruleTextColor) {
        this.ruleTextColor = ruleTextColor;
    }

    public Color getRuleTextColor() {
        return ruleTextColor;
    }


    public void setRuleTextAlignedOnBaseLine(boolean isRuleTextAlignedOnBaseLine) {
        this.isRuleTextAlignedOnBaseLine = isRuleTextAlignedOnBaseLine;
    }

    public boolean isRuleTextAlignedOnBaseLine() {
        return isRuleTextAlignedOnBaseLine;
    }


    public void setRuleShape(BoxShape ruleShape) {
        this.ruleShape = ruleShape;
    }

    public BoxShape getRuleShape() {
        return ruleShape;
    }


    public void setRuleBorderColor(Color ruleBorderColor) {
        this.ruleBorderColor = ruleBorderColor;
    }

    public Color getRuleBorderColor() {
        return ruleBorderColor;
    }


    public void setRuleFillColor(Color ruleFillColor) {
        this.ruleFillColor = ruleFillColor;
    }

    public Color getRuleFillColor() {
        return ruleFillColor;
    }


    public void setLiteralInsets(Insets literalInsets) {
        this.literalInsets = literalInsets;
    }

    public Insets getLiteralInsets() {
        return literalInsets;
    }


    public void setLiteralFont(Font literalFont) {
        this.literalFont = literalFont;
    }

    public Font getLiteralFont() {
        return literalFont;
    }

    public void setLiteralTextColor(Color literalTextColor) {
        this.literalTextColor = literalTextColor;
    }

    public Color getLiteralTextColor() {
        return literalTextColor;
    }


    public void setLiteralShape(BoxShape literalShape) {
        this.literalShape = literalShape;
    }

    public BoxShape getLiteralShape() {
        return literalShape;
    }


    public void setLiteralBorderColor(Color literalBorderColor) {
        this.literalBorderColor = literalBorderColor;
    }

    public Color getLiteralBorderColor() {
        return literalBorderColor;
    }


    public void setLiteralFillColor(Color literalFillColor) {
        this.literalFillColor = literalFillColor;
    }

    public Color getLiteralFillColor() {
        return literalFillColor;
    }

    public void setSpecialSequenceInsets(Insets specialSequenceInsets) {
        this.specialSequenceInsets = specialSequenceInsets;
    }

    public Insets getSpecialSequenceInsets() {
        return specialSequenceInsets;
    }


    public void setSpecialSequenceFont(Font specialSequenceFont) {
        this.specialSequenceFont = specialSequenceFont;
    }

    public Font getSpecialSequenceFont() {
        return specialSequenceFont;
    }


    public void setSpecialSequenceTextColor(Color specialSequenceTextColor) {
        this.specialSequenceTextColor = specialSequenceTextColor;
    }

    public Color getSpecialSequenceTextColor() {
        return specialSequenceTextColor;
    }


    public void setSpecialSequenceShape(BoxShape specialSequenceShape) {
        this.specialSequenceShape = specialSequenceShape;
    }

    public BoxShape getSpecialSequenceShape() {
        return specialSequenceShape;
    }


    public void setSpecialSequenceBorderColor(Color specialSequenceBorderColor) {
        this.specialSequenceBorderColor = specialSequenceBorderColor;
    }

    public Color getSpecialSequenceBorderColor() {
        return specialSequenceBorderColor;
    }


    public void setSpecialSequenceFillColor(Color specialSequenceFillColor) {
        this.specialSequenceFillColor = specialSequenceFillColor;
    }

    public Color getSpecialSequenceFillColor() {
        return specialSequenceFillColor;
    }

    public enum LineContinuationType {
        NONE, ELLIPSIS, PATH,
    }


    public void setLineContinuationType(LineContinuationType lineContinuationType) {
        if (lineContinuationType == null) {
            lineContinuationType = LineContinuationType.NONE;
        }
        this.lineContinuationType = lineContinuationType;
    }

    public LineContinuationType getLineContinuationType() {
        return lineContinuationType;
    }


    public void setLeftAligned(boolean isLeftAligned) {
        this.isLeftAligned = isLeftAligned;
    }

    public boolean isLeftAligned() {
        return isLeftAligned;
    }


    public void setStartElement(Railroad startElement) {
        this.startElement = startElement;
    }

    public Railroad getStartElement() {
        return startElement;
    }

    public void setEndElement(Railroad endElement) {
        this.endElement = endElement;
    }

    public Railroad getEndElement() {
        return endElement;
    }

    @Override
    public void setLayoutData(Railroad elem, LayoutData data) {
        this.layoutData.put(elem, data);
    }

    public LayoutData getLayoutData(Railroad elem) {
        return layoutData.get(elem);
    }
}
