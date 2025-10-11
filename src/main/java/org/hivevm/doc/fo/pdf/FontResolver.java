// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.pdf;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.URI;
import java.net.URL;
import org.apache.fop.fonts.apps.TTFReader;
import org.apache.fop.fonts.truetype.FontFileReader;
import org.apache.fop.fonts.truetype.TTFFile;
import org.apache.xmlgraphics.io.Resource;
import org.apache.xmlgraphics.io.ResourceResolver;
import org.hivevm.util.xml.StAX;

/**
 * The {@code FontResolver} class implements the {@link ResourceResolver} interface to provide
 * custom resource resolution functionality, including dynamic font-metric generation.
 */
class FontResolver implements ResourceResolver {

    /**
     * Resolves the given URI to a {@link Resource}. The method provides support for local files,
     * font metric generation for TTF XML descriptors, and other types of resources available via a
     * URL.
     */
    @Override
    public final Resource getResource(URI uri) throws IOException {
        if (uri.toString().startsWith("/")) {
            return new Resource(new FileInputStream(uri.toString()));
        }
        else if (uri.toString().endsWith("ttf.xml")) {
            var ttf = uri.toString().substring(0, uri.toString().length() - 4);
            return new Resource(FontResolver.createFontMetric(URI.create(ttf).toURL()));
        }
        return new Resource(uri.toURL().openStream());
    }

    @Override
    public final OutputStream getOutputStream(URI uri) throws IOException {
        throw new UnsupportedOperationException();
    }

    /**
     * Creates a font metric resource from the specified URL. The method reads a font file from the
     * given URL, generates font metrics in a supported format, and returns an input stream
     * containing the generated font metric resource.
     */
    private static InputStream createFontMetric(URL url) throws IOException {
        try (var iStream = url.openStream(); var oStream = new ByteArrayOutputStream()) {
            FontResolver.createFontMetric(iStream, oStream);
            return new Resource(new ByteArrayInputStream(oStream.toByteArray()));
        }
    }

    /**
     * Generates a font metric file in the form of an XML descriptor by reading the input TrueType
     * font (TTF) file and writing the resulting metric data to the specified output stream.
     */
    public static void createFontMetric(InputStream iStream, OutputStream oStream)
        throws IOException {
        var ttf = new TTFFile(true, true);
        ttf.readFont(new FontFileReader(iStream), null);
        if (ttf.isCFF())
            throw new UnsupportedOperationException(
                "OpenType fonts with CFF data are not supported, yet");

        var reader = new TTFReader();
        var doc = reader.constructFontXML(ttf, null, null, null, null, true, null);
        StAX.transform(oStream, doc);
    }
}
