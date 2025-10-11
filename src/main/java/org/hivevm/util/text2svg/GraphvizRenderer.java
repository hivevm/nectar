// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util.text2svg;

import guru.nidi.graphviz.engine.Format;
import guru.nidi.graphviz.engine.Graphviz;
import guru.nidi.graphviz.engine.GraphvizV8Engine;
import guru.nidi.graphviz.parse.Parser;
import net.sourceforge.plantuml.FileFormat;
import net.sourceforge.plantuml.FileFormatOption;
import net.sourceforge.plantuml.SourceStringReader;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.util.regex.Pattern;

public class GraphvizRenderer {

    private static final Pattern PLANTUML_PATTERN = Pattern.compile("^@start(\\w+)");
    private static final String PRAGMA_LAYOUT = "!pragma layout smetana";

    public static byte[] render(String text) throws IOException {
        Graphviz.useEngine(new GraphvizV8Engine());

        var matcher = PLANTUML_PATTERN.matcher(text);
        var isPlantUML = matcher.find();
        if (isPlantUML) {
            var start = "@start";
            var type = matcher.group(1);
            var uml = start + type + '\n' + PRAGMA_LAYOUT + text.substring(6 + type.length()) ;
            try (var stream = new ByteArrayOutputStream()) {
                var reader = new SourceStringReader(uml);
                reader.outputImage(stream, new FileFormatOption(FileFormat.SVG));
                return stream.toByteArray();
            }
        }

        try (var stream = new ByteArrayOutputStream()) {
            var graph = new Parser().read(text);
            Graphviz.fromGraph(graph)
                    .render(Format.SVG_STANDALONE)
                    .toOutputStream(stream);
            return stream.toByteArray();
        }
    }

    public static byte[] render(String text, String type) throws IOException {
        Graphviz.useEngine(new GraphvizV8Engine());

        var uml = "@start" + type + '\n' + PRAGMA_LAYOUT + '\n' + text + "\n@end\" + type + \"\n";
        try (var stream = new ByteArrayOutputStream()) {
            var reader = new SourceStringReader(uml);
            reader.outputImage(stream, new FileFormatOption(FileFormat.SVG));
            return stream.toByteArray();
        }
    }
}
