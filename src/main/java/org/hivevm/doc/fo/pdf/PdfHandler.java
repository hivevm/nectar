// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.pdf;

import java.util.List;
import org.apache.fop.apps.FOUserAgent;
import org.apache.fop.apps.io.InternalResourceResolver;
import org.apache.fop.fonts.EmbedFontInfo;
import org.apache.fop.fonts.FontCollection;
import org.apache.fop.render.intermediate.IFContext;
import org.apache.fop.render.intermediate.IFDocumentHandler;
import org.apache.fop.render.intermediate.IFDocumentHandlerConfigurator;
import org.apache.fop.render.pdf.PDFDocumentHandler;
import org.apache.fop.render.pdf.PDFDocumentHandlerMaker;
import org.apache.fop.render.pdf.PDFRendererConfig.PDFRendererConfigParser;
import org.apache.fop.render.pdf.PDFRendererConfigurator;

/**
 * The PdfHandler class is responsible for creating an IFDocumentHandler instance customized for PDF
 * rendering with embedded fonts. It extends the PDFDocumentHandlerMaker class to provide specific
 * PDF handling functionality.
 */
class PdfHandler extends PDFDocumentHandlerMaker {

    private final List<EmbedFontInfo> embedFonts;

    public PdfHandler(List<EmbedFontInfo> embedFonts) {
        this.embedFonts = embedFonts;
    }

    /**
     * Creates an instance of {@code IFDocumentHandler} customized for PDF rendering. This method
     * overrides the {@code makeIFDocumentHandler} method and provides a specialized implementation
     * for creating a PDF-specific document handler.
     */
    @Override
    public IFDocumentHandler makeIFDocumentHandler(IFContext context) {
        var handler = new PDFDocumentHandler(context) {

            @Override
            public IFDocumentHandlerConfigurator getConfigurator() {
                return new CustomPDFRendererConfigurator(getUserAgent());
            }
        };

        var userAgent = context.getUserAgent();
        if (userAgent.isAccessibilityEnabled()) {
            userAgent.setStructureTreeEventHandler(handler.getStructureTreeEventHandler());
        }
        return handler;
    }

    /**
     * The {@code CustomPDFRendererConfigurator} class is a specialized extension of
     * {@link PDFRendererConfigurator}, designed to configure a PDF renderer using custom settings.
     * It integrates a mechanism to include custom font collections based on a predefined list of
     * embedded fonts provided by its enclosing class, {@code PdfHandler}.
     */
    private class CustomPDFRendererConfigurator extends PDFRendererConfigurator {

        /**
         * Constructs a new instance of {@code CustomPDFRendererConfigurator}, which is a
         * specialized configuration class for the PDF renderer. It extends the
         * {@code PDFRendererConfigurator} to provide custom setup using the specified
         * {@code FOUserAgent}.
         */
        public CustomPDFRendererConfigurator(FOUserAgent userAgent) {
            super(userAgent, new PDFRendererConfigParser());
        }

        @Override
        protected final FontCollection getCustomFontCollection(InternalResourceResolver resolver,
            String mimeType) {
            return createCollectionFromFontList(resolver, PdfHandler.this.embedFonts);
        }
    }
}
