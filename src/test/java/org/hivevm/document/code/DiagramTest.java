// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document.code;

import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;

import java.io.*;
import java.util.Arrays;
import java.util.stream.Stream;

/**
 * The {@link DiagramTest} class.
 */
public abstract class DiagramTest {

    private final String type;
    private final StreamHandler handler;

    protected DiagramTest(String type, StreamHandler handler) {
        this.type = type;
        this.handler = handler;
    }

    protected final Stream<DynamicTest> dynamicTests(String type) {
        var source = new File("./doc");
        var filetype = "." + type;
        return Arrays.stream(source.listFiles(path -> path.getName().endsWith(filetype)))
                .map(file -> DynamicTest.dynamicTest(file.getName(), () -> {
                            var name = file.getName();
                            var filename = name.substring(0, name.length() - filetype.length());
                            var target = new File(file.getParent(), filename + ".svg");
                            try (var request = new FileInputStream(file);
                                 var response = new FileOutputStream(target)) {
                                handler.handleRequest(request, response);
                            }
                        }
                ));
    }

    @TestFactory
    protected Stream<DynamicTest> dynamicTests() {
        return dynamicTests(type);
    }

    @FunctionalInterface
    protected interface StreamHandler {
        void handleRequest(InputStream inputStream, OutputStream outputStream) throws IOException;
    }
}