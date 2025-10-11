// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

import java.io.File;
import java.net.MalformedURLException;
import java.net.URI;
import java.util.Stack;
import java.util.regex.Pattern;
import org.hivevm.util.DataUri;
import org.hivevm.util.xml.StAX;
import org.hivevm.util.xml.StAX.Attributes;


/**
 * The {@link TemplateParser} class.
 */
class TemplateParser implements StAX.Handler {

    private final TemplateBuilder builder;
    private final File            workingDir;

    private final Stack<Object>  items;
    private final Stack<PageSet> pageSets;

    private TemplateFont font;


    /**
     * Constructs an instance of {@link TemplateParser}.
     */
    public TemplateParser(TemplateBuilder builder, File workingDir) {
        this.builder = builder;
        this.workingDir = workingDir;
        this.items = new Stack<>();
        this.pageSets = new Stack<>();
    }

    /**
     * Get {@link TemplateBuilder}.
     */
    protected final TemplateBuilder getBuilder() {
        return this.builder;
    }

    /**
     * Gets the working directory.
     */
    protected final File getWorkingDir() {
        return this.workingDir;
    }

    /**
     * Get top item.
     *
     * @param clazz
     */
    @SuppressWarnings("unchecked")
    protected final <I> I getItem(Class<I> clazz) {
        return (I) this.items.peek();
    }

    /**
     * Adds an item.
     *
     * @param item
     */
    protected final <I> I addItem(I item) {
        this.items.push(item);
        return item;
    }

    /**
     * Release the current item.
     */
    protected final void releaseItem() {
        this.items.pop();
    }

    /**
     * Adds a font metric to the current font.
     *
     * @param path
     * @param bold
     * @param italic
     */
    protected final void addFontMetric(String path, boolean bold, boolean italic) {
        URI uri = DataUri.toURI(path, getWorkingDir());
        this.font.addMetric(uri, bold, italic);
    }


    @Override
    public void handleEvent(String tagName, Attributes attrs) {
        switch (tagName) {
            case "template":
                getBuilder().setSize(attrs.get("width"), attrs.get("height"));
                getBuilder().setFont(attrs.get("font-family"));
                break;

            case "font":
                if (attrs.isSet("codepoints")) {
                    try {
                        URI codepoints = DataUri.toURI(attrs.get("codepoints"), null);
                        getBuilder().addFontSymbols(attrs.get("name"), codepoints.toURL());
                    } catch (MalformedURLException e) {
                    }
                }
                this.font = getBuilder().addFont(attrs.get("name"));
                break;

            case "font-metric":
                addFontMetric(attrs.get("uri"), attrs.getBool("bold"), attrs.getBool("italic"));
                break;

            case "style":
                PageStyle style = new PageStyle(Pattern.compile("^" + attrs.get("match") + "$"));
                getBuilder().addStyle(style);
                addItem(style);

                attrs.onAttribute("color", style::setColor);
                attrs.onAttribute("background", v -> style.setBackground(Background.of(v, getWorkingDir())));

                attrs.onAttribute("font-size", style::setFontSize);
                attrs.onAttribute("font-style", style::setFontStyle);
                attrs.onAttribute("font-weight", style::setFontWeight);
                attrs.onAttribute("font-family", style::setFontFamily);
                attrs.onAttribute("text-align", style::setTextAlign);
                attrs.onAttribute("line-height", style::setLineHeight);

                attrs.onAttribute("span", style::setSpan);
                attrs.onAttribute("keep-with-next", style::setKeepWithNext);
                attrs.onAttribute("space-after", style::setSpaceAfter);
                attrs.onAttribute("space-before", style::setSpaceBefore);

                attrs.onAttribute("border", style::setBorder);
                attrs.onAttribute("border-top", style::setBorderTop);
                attrs.onAttribute("border-left", style::setBorderLeft);
                attrs.onAttribute("border-right", style::setBorderRight);
                attrs.onAttribute("border-bottom", style::setBorderBottom);
                attrs.onAttribute("border-radius", style::setBorderRadius);

                attrs.onAttribute("margin", style::setMargin);
                attrs.onAttribute("margin-top", style::setMarginTop);
                attrs.onAttribute("margin-left", style::setMarginLeft);
                attrs.onAttribute("margin-right", style::setMarginRight);
                attrs.onAttribute("margin-bottom", style::setMarginBottom);

                attrs.onAttribute("padding", style::setPadding);
                attrs.onAttribute("padding-top", style::setPaddingTop);
                attrs.onAttribute("padding-left", style::setPaddingLeft);
                attrs.onAttribute("padding-right", style::setPaddingRight);
                attrs.onAttribute("padding-bottom", style::setPaddingBottom);

                attrs.onAttribute("break-before", style::setBreakBefore);
                attrs.onAttribute("break-after", style::setBreakAfter);
                break;

            case "page":
                Page page = getBuilder().addPage(attrs.get("name"));
                page.setPageSize(
                        attrs.get("width", getBuilder().getWidth()),
                        attrs.get("height", getBuilder().getHeight()));
                addItem(page);

                attrs.onAttribute("background",
                        v -> page.background = Background.of(v, getWorkingDir()));

                if (attrs.isSet("column-count")) {
                    page.setColumns(attrs.get("column-count"), attrs.get("column-gap"));
                }

                attrs.onAttribute("margin-top", p -> page.setMarginTop(p));
                attrs.onAttribute("margin-left", p -> page.setMarginLeft(p));
                attrs.onAttribute("margin-right", p -> page.setMarginRight(p));
                attrs.onAttribute("margin-bottom", p -> page.setMarginBottom(p));
                attrs.onAttribute("margin", p -> {
                    String[] margin = p.split(",");
                    if (margin.length == 1) {
                        margin = new String[]{margin[0], margin[0], margin[0], margin[0]};
                    } else if (margin.length == 2) {
                        margin = new String[]{margin[0], margin[1], margin[0], margin[1]};
                    }

                    page.setMarginTop(margin[0]);
                    page.setMarginLeft(margin[3]);
                    page.setMarginRight(margin[1]);
                    page.setMarginBottom(margin[2]);
                });

                attrs.onAttribute("padding-top", p -> page.setPaddingTop(p));
                attrs.onAttribute("padding-left", p -> page.setPaddingLeft(p));
                attrs.onAttribute("padding-right", p -> page.setPaddingRight(p));
                attrs.onAttribute("padding-bottom", p -> page.setPaddingBottom(p));
                attrs.onAttribute("padding", p -> {
                    String[] padding = p.split(",");
                    if (padding.length == 1) {
                        padding = new String[]{padding[0], padding[0], padding[0], padding[0]};
                    } else if (padding.length == 2) {
                        padding = new String[]{padding[0], padding[1], padding[0], padding[1]};
                    }

                    page.setPaddingTop(padding[0]);
                    page.setPaddingLeft(padding[3]);
                    page.setPaddingRight(padding[1]);
                    page.setPaddingBottom(padding[2]);
                });
                break;

            case "region":
                page = getItem(Page.class);
                PageRegion region = page.getRegion(attrs.get("position"));
                region.setExtent(attrs.get("extent"));
                addItem(region);
                break;

            case "page-entry":
                PageMatch match = attrs.isSet("match")
                        ? PageMatch.valueOf(attrs.get("match")) : PageMatch.Any;
                PageSet pageSet = this.pageSets.peek();
                pageSet.addPage(getBuilder().getTemplate(attrs.get("name")), match);
                break;

            case "page-set":
                this.pageSets.add(getBuilder().addPageSet(attrs.get("name")));
                break;

            case "image":
                URI uri = DataUri.toURI(attrs.get("uri"), getWorkingDir());
                PageImage image = new PageImage(uri);
                getItem(PageContainer.class).addItem(image);

                attrs.onAttribute("top", image::setTop);
                attrs.onAttribute("left", image::setLeft);
                attrs.onAttribute("right", image::setRight);
                attrs.onAttribute("bottom", image::setBottom);
                attrs.onAttribute("width", image::setWidth);
                attrs.onAttribute("height", image::setHeight);

                addItem(image);
                break;

            case "column":
                PageColumn container = new PageColumn();
                getItem(PageContainer.class).addItem(container);

                attrs.onAttribute("top", container::setTop);
                attrs.onAttribute("left", container::setLeft);
                attrs.onAttribute("right", container::setRight);
                attrs.onAttribute("bottom", container::setBottom);

                attrs.onAttribute("color", container::setColor);
                attrs.onAttribute("background",
                        v -> container.setBackground(Background.of(v, getWorkingDir())));

                attrs.onAttribute("font-size", container::setFontSize);
                attrs.onAttribute("font-style", container::setFontStyle);
                attrs.onAttribute("font-weight", container::setFontWeight);
                attrs.onAttribute("font-family", container::setFontFamily);
                attrs.onAttribute("text-align", container::setTextAlign);
                attrs.onAttribute("line-height", container::setLineHeight);

                attrs.onAttribute("span", container::setSpan);
                attrs.onAttribute("keep-with-next", container::setKeepWithNext);
                attrs.onAttribute("space-after", container::setSpaceAfter);
                attrs.onAttribute("space-before", container::setSpaceBefore);

                attrs.onAttribute("border", container::setBorder);
                attrs.onAttribute("border-top", container::setBorderTop);
                attrs.onAttribute("border-left", container::setBorderLeft);
                attrs.onAttribute("border-right", container::setBorderRight);
                attrs.onAttribute("border-bottom", container::setBorderBottom);
                attrs.onAttribute("border-radius", container::setBorderRadius);

                attrs.onAttribute("margin", container::setMargin);
                attrs.onAttribute("margin-top", container::setMarginTop);
                attrs.onAttribute("margin-left", container::setMarginLeft);
                attrs.onAttribute("margin-right", container::setMarginRight);
                attrs.onAttribute("margin-bottom", container::setMarginBottom);

                attrs.onAttribute("padding", container::setPadding);
                attrs.onAttribute("padding-top", container::setPaddingTop);
                attrs.onAttribute("padding-left", container::setPaddingLeft);
                attrs.onAttribute("padding-right", container::setPaddingRight);
                attrs.onAttribute("padding-bottom", container::setPaddingBottom);

                attrs.onAttribute("break-before", container::setBreakBefore);
                attrs.onAttribute("break-after", container::setBreakAfter);
                addItem(container);
                break;

            case "row":
                PageRow text = new PageRow();
                attrs.onAttribute("color", text::setColor);
                attrs.onAttribute("font-size", text::setFontSize);
                attrs.onAttribute("font-style", text::setFontStyle);
                attrs.onAttribute("font-weight", text::setFontWeight);
                attrs.onAttribute("font-family", text::setFontFamily);
                attrs.onAttribute("text-align", text::setTextAlign);
                attrs.onAttribute("line-height", text::setLineHeight);

                attrs.onAttribute("top", text::setTop);
                attrs.onAttribute("left", text::setLeft);
                attrs.onAttribute("right", text::setRight);
                attrs.onAttribute("bottom", text::setBottom);

                getItem(PageContainer.class).addItem(text);
                addItem(text);
                break;

            case "span":
                PageTextInline inline = new PageTextInline();
                attrs.onAttribute("color", inline::setColor);
                attrs.onAttribute("font-size", inline::setFontSize);
                attrs.onAttribute("font-style", inline::setFontStyle);
                attrs.onAttribute("font-weight", inline::setFontWeight);
                attrs.onAttribute("font-family", inline::setFontFamily);
                attrs.onAttribute("text-align", inline::setTextAlign);
                attrs.onAttribute("line-height", inline::setLineHeight);

                getItem(PageRow.class).addTextItem(inline);
                addItem(inline);
                break;

            case "br":
                PageBreak line = new PageBreak(attrs.get("size"), attrs.get("style"), attrs.get("color"));
                attrs.onAttribute("top", line::setTop);
                attrs.onAttribute("left", line::setLeft);
                attrs.onAttribute("right", line::setRight);
                attrs.onAttribute("bottom", line::setBottom);

                getItem(PageContainer.class).addItem(line);
                addItem(line);
                break;
        }
    }

    @Override
    public void handleEvent(String name, String content) {
        switch (name) {
            case "span":
                getItem(PageTextInline.class).setText(content);
                releaseItem();
                break;

            case "row":
                getItem(PageRow.class).addText(content);
                releaseItem();
                break;

            case "template":
            case "style":
            case "region":
            case "column":
            case "image":
            case "br":
                releaseItem();
                break;
        }
    }
}
