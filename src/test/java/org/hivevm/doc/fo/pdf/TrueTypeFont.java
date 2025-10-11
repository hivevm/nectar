// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.pdf;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.FilenameFilter;
import java.util.Objects;

public abstract class TrueTypeFont {

    private static final File           FONT_PATH   = new File("src/main/resources/fonts/victor");
    private static final FilenameFilter FONT_FILTER = (d, n) -> n.toLowerCase().endsWith(".ttf");

    public static void main(String[] args) throws Exception {
        var fonts = Objects.requireNonNull(TrueTypeFont.FONT_PATH.listFiles(FONT_FILTER));
        for (var font : fonts) {
            var metric = new File(TrueTypeFont.FONT_PATH, String.format("%s.xml", font.getName()));
            try (var iStream = new FileInputStream(font.getAbsolutePath());
                 var oStream = new FileOutputStream(metric.getAbsolutePath())) {
                FontResolver.createFontMetric(iStream, oStream);
            }
        }
    }
}
