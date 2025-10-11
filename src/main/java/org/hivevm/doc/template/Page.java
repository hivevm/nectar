// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;
import org.hivevm.util.Unit;

/**
 * The {@link Page} class.
 */
public class Page implements PageContainer {

    private final String name;

    private Unit pageWidth  = Unit.NIL;
    private Unit pageHeight = Unit.NIL;

    private int  columns   = 1;
    private Unit columnGap = Unit.NIL;

    private Unit paddingTop    = Unit.NIL;
    private Unit paddingLeft   = Unit.NIL;
    private Unit paddingRight  = Unit.NIL;
    private Unit paddingBottom = Unit.NIL;

    private Unit marginTop    = Unit.NIL;
    private Unit marginLeft   = Unit.NIL;
    private Unit marginRight  = Unit.NIL;
    private Unit marginBottom = Unit.NIL;

    private final PageRegion regionBefore;
    private final PageRegion regionAfter;
    private final PageRegion regionStart;
    private final PageRegion regionEnd;

    public Background background;


    public final List<PageRenderer.PageRenderable> items = new ArrayList<>();

    /**
     * Constructs an instance of {@link Page}.
     *
     * @param name
     */
    public Page(String name) {
        this.name = name;
        this.regionBefore = new PageRegion(this, PageRegion.Region.BEFORE);
        this.regionStart = new PageRegion(this, PageRegion.Region.START);
        this.regionEnd = new PageRegion(this, PageRegion.Region.END);
        this.regionAfter = new PageRegion(this, PageRegion.Region.AFTER);
    }

    /**
     * Gets the name.
     */
    public final String getName() {
        return this.name;
    }

    /**
     * Set the template size.
     *
     * @param width
     * @param height
     */
    public void setPageSize(String width, String height) {
        this.pageWidth = Unit.parse(width);
        this.pageHeight = Unit.parse(height);
    }

    /**
     * Set the columns of the template.
     *
     * @param count
     * @param gap
     */
    public void setColumns(String count, String gap) {
        this.columns = Integer.parseInt(count);
        this.columnGap = Unit.parse(gap);
    }

    /**
     * Add a region to the page.
     *
     * @param position
     */
    public PageRegion getRegion(String position) {
        PageRegion region = switch (position.toLowerCase()) {
            case "top" -> regionBefore;
            case "left" -> regionStart;
            case "right" -> regionEnd;
            case "bottom" -> regionAfter;
            default -> regionBefore;
        };
        return region;
    }

    public String pageWidth() {
        return this.pageWidth.asString();
    }

    public String pageHeight() {
        return this.pageHeight.asString();
    }

    public String columnCount() {
        return String.valueOf(this.columns);
    }

    public String columnGap() {
        return this.columnGap.asString();
    }

    public String paddingTop() {
        return this.paddingTop.asString();
    }

    public String paddingLeft() {
        return this.paddingLeft.asString();
    }

    public String paddingRight() {
        return this.paddingRight.asString();
    }

    public String paddingBottom() {
        return this.paddingBottom.asString();
    }

    public String marginTop() {
        return this.marginTop.asString();
    }

    public String marginLeft() {
        return this.marginLeft.asString();
    }

    public String marginRight() {
        return this.marginRight.asString();
    }

    public String marginBottom() {
        return this.marginBottom.asString();
    }

    void setPaddingTop(String value) {
        this.paddingTop = Unit.parse(value);
    }

    void setPaddingLeft(String value) {
        this.paddingLeft = Unit.parse(value);
    }

    void setPaddingRight(String value) {
        this.paddingRight = Unit.parse(value);
    }

    void setPaddingBottom(String value) {
        this.paddingBottom = Unit.parse(value);
    }

    void setMarginTop(String value) {
        this.marginTop = Unit.parse(value);
    }

    void setMarginLeft(String value) {
        this.marginLeft = Unit.parse(value);
    }

    void setMarginRight(String value) {
        this.marginRight = Unit.parse(value);
    }

    void setMarginBottom(String value) {
        this.marginBottom = Unit.parse(value);
    }

    @Override
    public final void addItem(PageRenderer.PageRenderable child) {
        this.items.add(child);
    }

    public final void forEachRegion(Consumer<PageRegion> action) {
        List.of(regionBefore, regionAfter, regionStart, regionEnd).forEach(action);
    }
}
