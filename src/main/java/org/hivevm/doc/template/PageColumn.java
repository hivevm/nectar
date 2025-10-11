// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

import java.util.ArrayList;
import java.util.List;

/**
 * The {@link PageColumn} defines a generic interface for all template items.
 */
public class PageColumn extends PageRenderer.PageRenderable implements PageContainer {

    public String     color;
    public Background background;

    public String fontSize;
    public String fontStyle;
    public String fontWeight;
    public String fontFamily;

    public String span;
    public String textAlign;
    public String lineHeight;
    public String keepWithNext;

    public String spaceBefore;
    public String spaceAfter;

    public String borderRadius;
    public String borderTop;
    public String borderLeft;
    public String borderRight;
    public String borderBottom;

    public String marginTop;
    public String marginLeft;
    public String marginRight;
    public String marginBottom;

    public String paddingTop;
    public String paddingLeft;
    public String paddingRight;
    public String paddingBottom;

    public String breakBefore;
    public String breakAfter;

    public final List<PageRenderer.PageRenderable> items = new ArrayList<>();

    /**
     * Constructs an instance of {@link PageColumn}.
     */
    public PageColumn() {
        this.textAlign = "left";
        this.lineHeight = "1.5em";
    }

    @Override
    public final void addItem(PageRenderer.PageRenderable child) {
        this.items.add(child);
    }

    public final void setColor(String color) {
        this.color = color;
    }

    public final void setBackground(Background background) {
        this.background = background;
    }

    public final void setSpan(String span) {
        this.span = span;
    }

    public final void setFontSize(String fontSize) {
        this.fontSize = fontSize;
    }

    public final void setFontStyle(String fontStyle) {
        this.fontStyle = fontStyle;
    }

    public final void setFontWeight(String fontWeight) {
        this.fontWeight = fontWeight;
    }

    public final void setFontFamily(String fontFamily) {
        this.fontFamily = fontFamily;
    }

    public final void setTextAlign(String textAlign) {
        this.textAlign = textAlign;
    }

    public final void setLineHeight(String lineHeight) {
        this.lineHeight = lineHeight;
    }

    public final void setKeepWithNext(String keepWithNext) {
        this.keepWithNext = keepWithNext;
    }

    public final void setSpaceBefore(String spaceBefore) {
        this.spaceBefore = spaceBefore;
    }

    public final void setSpaceAfter(String spaceAfter) {
        this.spaceAfter = spaceAfter;
    }

    public final void setBorderRadius(String radius) {
        this.borderRadius = radius;
    }

    public final void setBorder(String value) {
        setBorderTop(value);
        setBorderLeft(value);
        setBorderRight(value);
        setBorderBottom(value);
    }

    public final void setBorderTop(String value) {
        this.borderTop = value;
    }

    public final void setBorderLeft(String value) {
        this.borderLeft = value;
    }

    public final void setBorderRight(String value) {
        this.borderRight = value;
    }

    public final void setBorderBottom(String value) {
        this.borderBottom = value;
    }

    public final void setMargin(String value) {
        String[] items = value.split(",");
        setMarginTop(items.length == 2 ? items[0] : items.length == 4 ? items[1] : value);
        setMarginLeft(items.length == 2 ? items[1] : items.length == 4 ? items[0] : value);
        setMarginRight(items.length == 2 ? items[1] : items.length == 4 ? items[2] : value);
        setMarginBottom(items.length == 2 ? items[0] : items.length == 4 ? items[3] : value);
    }

    public final void setMarginTop(String value) {
        this.marginTop = value;
    }

    public final void setMarginLeft(String value) {
        this.marginLeft = value;
    }

    public final void setMarginRight(String value) {
        this.marginRight = value;
    }

    public final void setMarginBottom(String value) {
        this.marginBottom = value;
    }

    public final void setPadding(String value) {
        String[] items = value.split(",");
        setPaddingTop(items.length == 2 ? items[0] : items.length == 4 ? items[1] : value);
        setPaddingLeft(items.length == 2 ? items[1] : items.length == 4 ? items[0] : value);
        setPaddingRight(items.length == 2 ? items[1] : items.length == 4 ? items[2] : value);
        setPaddingBottom(items.length == 2 ? items[0] : items.length == 4 ? items[3] : value);
    }

    public final void setPaddingTop(String value) {
        this.paddingTop = value;
    }

    public final void setPaddingLeft(String value) {
        this.paddingLeft = value;
    }

    public final void setPaddingRight(String value) {
        this.paddingRight = value;
    }

    public final void setPaddingBottom(String value) {
        this.paddingBottom = value;
    }

    public final void setBreakBefore(String value) {
        this.breakBefore = value;
    }

    public final void setBreakAfter(String value) {
        this.breakAfter = value;
    }

    @Override
    public final <B> void accept(PageRenderer<B> visitor, B block) {
        visitor.visit(this, block);
    }
}
