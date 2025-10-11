// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.pdf;

import java.io.OutputStream;
import java.net.URI;
import java.util.ArrayList;
import java.util.Collections;
import org.apache.fop.apps.FOPException;
import org.apache.fop.apps.Fop;
import org.apache.fop.apps.FopFactoryBuilder;
import org.apache.fop.fonts.EmbedFontInfo;
import org.apache.fop.fonts.Font;
import org.apache.fop.fonts.FontTriplet;
import org.apache.fop.fonts.FontUris;
import org.apache.xmlgraphics.util.MimeConstants;
import org.hivevm.doc.template.Template;

public class PdfBuilder {

    private final URI      uri;
    private final Template template;

    public PdfBuilder(URI uri, Template template) {
        this.uri = uri;
        this.template = template;
    }


    public Fop build(OutputStream output) throws FOPException {
        var embedded = new ArrayList<EmbedFontInfo>();
        template.getFonts().forEach(f -> {
            for (var metric : f.getMetrics()) {
                var font = new FontUris(metric.uri(), URI.create(metric.uri().toString() + ".xml"));
                var style = metric.italic() ? Font.STYLE_ITALIC : Font.STYLE_NORMAL;
                var weight = metric.bold() ? Font.WEIGHT_BOLD : Font.WEIGHT_NORMAL;
                var triplets = Collections.singletonList(
                    new FontTriplet(f.getName(), style, weight));
                embedded.add(new EmbedFontInfo(font, true, false, triplets, null));
            }
        });

        var builder = new FopFactoryBuilder(uri, new FontResolver());
        builder.setStrictFOValidation(true).setStrictUserConfigValidation(true);
        builder.setSourceResolution(72).setTargetResolution(72);
        builder.setPageWidth(template.getWidth()).setPageHeight(template.getHeight());
        builder.setPreferRenderer(true);

        var factory = builder.build();
        factory.getRendererFactory().addDocumentHandlerMaker(new PdfHandler(embedded));

        // a user agent is needed for transformation
        var foUserAgent = factory.newFOUserAgent();
        // foUserAgent.getRendererOptions().put("encryption-length", 128);
        // foUserAgent.getRendererOptions().put("user-password", "test");
        // foUserAgent.getRendererOptions().put("owner-password", "test");
        // foUserAgent.getRendererOptions().put("noprint", "true");
        // foUserAgent.getRendererOptions().put("nocopy", "true");
        // foUserAgent.getRendererOptions().put("noedit", "true");
        // foUserAgent.getRendererOptions().put("noannotations", "true");
        // foUserAgent.getRendererOptions().put("nofillinforms", "true");
        // foUserAgent.getRendererOptions().put("noaccesscontent", "true");
        // foUserAgent.getRendererOptions().put("noassembledoc", "true");
        // foUserAgent.getRendererOptions().put("noprinthq", "true");

        // Step 1: Construct a FopFactory by specifying a reference to the
        // configuration file (reuse if you plan to render multiple documents!)

        // Step 2: Construct fop with desired output format
        // MIME_PDF = "application/pdf";
        return foUserAgent.newFop(MimeConstants.MIME_PDF, output);
    }
}
