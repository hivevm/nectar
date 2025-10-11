// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

import org.apache.fop.apps.FOPException;
import org.hivevm.commonmark.Markdown;
import org.hivevm.doc.fo.pdf.PdfBuilder;
import org.hivevm.doc.template.Template;
import org.hivevm.document.fo.FoRenderer;
import org.hivevm.util.StreamHandler;
import org.hivevm.util.StreamPipeline;
import org.hivevm.util.xml.StAX;
import org.junit.jupiter.api.Test;

import javax.xml.stream.XMLStreamException;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;

/// Test class for verifying the functionality of the Markdown parsing process. This class validates
/// if Markdown content can be read from a file, parsed correctly, and visited using the
/// {@code MarkdownVisitor}.
///
/// The test focuses on the following:
///
/// 1. Reading Markdown content from a file.
/// 2. Parsing the Markdown content into a {@code Node} structure using the {@code Markdown.parse()}
/// method.
/// 3. Visiting the parsed Markdown nodes using a custom {@code MarkdownVisitor}.
public class MarkdownParserTest {

    private static final String NAME = "README";
    private static final TestContext CONTEXT = TestContext.local();

    @Test
    public void testMarkdown() throws IOException {
        var mdFile = new File(CONTEXT.workingDir(), NAME + ".md");
        Markdown.parse(mdFile).accept(new MarkdownParser());

        try (var stream = new FileInputStream(mdFile)) {
            var doc = MarkdownParser.parse(stream);
            System.out.println("");
        }
    }

    @Test
    public void testPDF() throws Exception {
        var template = Template.getDefault();
        var mdFile = new File(CONTEXT.workingDir(), NAME + ".md");
        var foFile = new File(CONTEXT.target(), NAME + ".fo");
        var pdfFile = new File(CONTEXT.target(), NAME + ".pdf");

        var builder = StreamPipeline.builder()
                // Generates a FO document from the Markdown content.
                .addHandler((input, output) -> {
                    var document = MarkdownParser.parse(input);
                    try (var writer = new FoRenderer(output, template)) {
                        writer.render(document);
                    } catch (XMLStreamException e) {
                        throw new IOException(e);
                    }
                })
                // Generates a PDF file from the FO document.
                .addHandler(getPdfHandler(template));

        try (var in = new FileInputStream(mdFile);
             var out = new FileOutputStream(pdfFile);
             var pipeline = builder.build()) {
            pipeline.handleRequest(in, out);
        }
    }

    private static StreamHandler getFileHandler(Template template) {
        return (input, output) -> {
            output.write(input.readAllBytes());
        };
    }

    private static StreamHandler getPdfHandler(Template template) {
        return (input, output) -> {
            try {
                var pdf = new PdfBuilder(CONTEXT.workingDir().toURI(), template);
                var fop = pdf.build(output);
                StAX.transform(input, fop.getDefaultHandler());
            } catch (FOPException e) {
                throw new IOException(e);
            }
        };
    }
}