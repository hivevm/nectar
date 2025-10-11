// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.pdf;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import org.apache.fop.apps.FOPException;
import org.hivevm.doc.template.Template;
import org.hivevm.util.lambda.RequestStreamHandler;
import org.hivevm.util.xml.StAX;

public class PdfRenderer implements RequestStreamHandler {

    private final Template template;

    public PdfRenderer(Template template) {
        this.template = template;
    }

    public void handleRequest(InputStream input, OutputStream output, File context)
        throws IOException {
        var builder = new PdfBuilder(context.toURI(), template);
        try {
            var fop = builder.build(output);
            StAX.transform(input, fop.getDefaultHandler());
        } catch (FOPException e) {
            throw new IOException(e);
        }
    }
}
