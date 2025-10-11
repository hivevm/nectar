// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.commonmark.image;

import org.commonmark.parser.Parser;
import org.commonmark.renderer.html.HtmlRenderer;

/**
 * The ImageExtension class provides functionality to extend both the CommonMark Parser and
 * HtmlRenderer to support custom image attributes. It implements the Parser.ParserExtension and
 * HtmlRenderer.HtmlRendererExtension interfaces.
 */
public record ImageExtension() implements Parser.ParserExtension,
    HtmlRenderer.HtmlRendererExtension {

    @Override
    public void extend(Parser.Builder parserBuilder) {
        parserBuilder.customDelimiterProcessor(new ImageProcessor());
    }

    @Override
    public void extend(HtmlRenderer.Builder rendererBuilder) {
        rendererBuilder.attributeProviderFactory(context -> ImageProvider.create());
    }
}
