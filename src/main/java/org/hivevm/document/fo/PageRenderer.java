// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause
package org.hivevm.document.fo;

import org.hivevm.doc.fo.writer.FoBasicLink;
import org.hivevm.doc.fo.writer.FoBlock;
import org.hivevm.doc.fo.writer.FoExternalGraphic;
import org.hivevm.doc.template.PageBreak;
import org.hivevm.doc.template.PageColumn;
import org.hivevm.doc.template.PageImage;
import org.hivevm.doc.template.PageRow;
import org.hivevm.util.DataUri;

import java.util.Properties;
import java.util.function.Supplier;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

class PageRenderer implements org.hivevm.doc.template.PageRenderer<FoBlock> {

    private static final Pattern LINK = Pattern.compile("\\[([^\\]]+)\\]\\(([^\\)]+)\\)",
            Pattern.CASE_INSENSITIVE);
    private static final Pattern ENV = Pattern.compile("\\{\\{\\$([^}]+)\\}\\}",
            Pattern.CASE_INSENSITIVE);

    private final Properties properties;


    public PageRenderer(Properties properties) {
        this.properties = properties;
        this.properties.put("PAGE_NUMBER", "<fo:page-number/>");
    }

    @Override
    public final void visit(PageColumn item, FoBlock block) {
        block.setSpan(item.span);
        block.setMargin(item.getLeft(), item.getRight(), item.getTop(), item.getBottom());
        block.setSpan(item.span);

        block.setColor(item.color);
        if (item.background != null) {
            if (item.background.isImage()) {
                block.setBackground(item.background.get(), "no-repeat");
            } else {
                block.setBackgroundColor(item.background.get());
            }
        }
        block.setFontSize(item.fontSize);
        block.setFontStyle(item.fontStyle);
        block.setFontWeight(item.fontWeight);
        block.setFontFamily(item.fontFamily);

        block.setTextAlign(item.textAlign);
        block.setLineHeight(item.lineHeight);

        item.items.forEach(c -> c.accept(this, block.addBlock()));
    }

    @Override
    public final void visit(PageRow item, FoBlock block) {
        block.setColor(item.color);
        block.setFontSize(item.fontSize);
        block.setFontStyle(item.fontStyle);
        block.setFontWeight(item.fontWeight);
        block.setFontFamily(item.fontFamily);

        block.setTextAlign(item.textAlign);
        block.setLineHeight(item.lineHeight);
        block.setPadding(item.getLeft(), item.getRight(), item.getTop(), item.getBottom());

        item.forEachInline(i -> {
            FoBlock inline = block.addInline();
            inline.setColor(i.color);
            inline.setFontSize(i.fontSize);
            inline.setFontStyle(i.fontStyle);
            inline.setFontWeight(i.fontWeight);
            inline.setFontFamily(i.fontFamily);
            inline.setTextAlign(i.textAlign);
            inline.setLineHeight(i.lineHeight);

            addAndReplaceText(i.text, inline);
        });
    }

    @Override
    public final void visit(PageImage item, FoBlock block) {
        var graphic = new FoExternalGraphic(block);
        graphic.setURL(DataUri.loadImage(item.uri));
    }

    @Override
    public final void visit(PageBreak item, FoBlock block) {
        block.setBorderBottom(item.size, item.style, item.color);
    }


    private void addAndReplaceText(String text, FoBlock block) {
        int offset = 0;

        text = replaceText(text);
        Matcher matcher = PageRenderer.LINK.matcher(text);
        while (matcher.find()) {
            if (offset < matcher.start()) {
                FoBlock inline = FoBlock.inline(block);
                inline.addContent(text.substring(offset, matcher.start()));
            }

            FoBasicLink link = new FoBasicLink(block);
            link.setDestination("mailto:" + matcher.group(2));
            link.addText(matcher.group(1));

            offset = matcher.end();
        }
        if (offset < text.length()) {
            String content = text.substring(offset);
            FoBlock inline = block.addInline();
            if (content.startsWith("<fo:page-number")) {
                inline.create("page-number");
            } else {
                inline.addContent(content);
            }
        }
    }

    private String replaceText(String text) {
        int offset = 0;
        StringBuilder builder = new StringBuilder();

        Matcher matcher = PageRenderer.ENV.matcher(text);
        while (matcher.find()) {
            Object value = properties.get(matcher.group(1));
            builder.append(text, offset, matcher.start());
            builder.append(value instanceof Supplier ? ((Supplier<String>) value).get() : value);
            offset = matcher.end();
        }
        builder.append(text.substring(offset));
        return builder.toString();
    }
}