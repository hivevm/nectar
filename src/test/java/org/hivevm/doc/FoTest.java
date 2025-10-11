// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc;

import org.hivevm.doc.fo.pdf.PdfRenderer;
import org.hivevm.doc.template.Template;
import org.hivevm.document.TestContext;
import org.hivevm.util.DataUri;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

import java.io.File;
import java.io.FileOutputStream;
import java.util.Arrays;
import java.util.stream.Stream;

/**
 * The {@link FoTest} class.
 */
public class FoTest {

    @TestFactory
    Stream<DynamicTest> dynamicFoTests() throws Exception {
        var target = TestContext.defaults().target();
        var template = Template.getDefault();
        var files = new String[]{"test", "test-demo"};
        return Arrays.stream(files).map(filename -> DynamicTest.dynamicTest(filename, () -> {
                var path = String.format(":%s.fo.xml", filename);
                var output = new File(target, filename + ".pdf");
                try (var oStream = new FileOutputStream(output);
                    var iStream = DataUri.toInputStream(path, target)) {
                    var handler = new PdfRenderer(template);
                    handler.handleRequest(iStream, oStream, target);
                }
            })
        );
    }
}