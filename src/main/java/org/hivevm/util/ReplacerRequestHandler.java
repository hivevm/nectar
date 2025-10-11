// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util;

import java.io.BufferedReader;
import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.io.OutputStreamWriter;
import java.io.Writer;
import java.util.Map;
import java.util.stream.Collectors;
import org.hivevm.util.lambda.RequestStreamHandler;

/**
 * The {@link ReplacerRequestHandler} class.
 */
public class ReplacerRequestHandler implements RequestStreamHandler {

    private final Replacer replacer;

    public ReplacerRequestHandler(Map<String, String> properties) {
        this.replacer = new Replacer(properties);
    }

    public void handleRequest(InputStream input, OutputStream output, File context) throws IOException {
        try (BufferedReader reader = new BufferedReader(new InputStreamReader(input))) {
            try (Writer writer = new OutputStreamWriter(output)) {
                writer.write(replacer.replaceAll(reader.lines().collect(Collectors.joining("\n"))));
            }
        }
    }
}