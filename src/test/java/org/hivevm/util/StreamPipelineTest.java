// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;

public class StreamPipelineTest {

    @Test
    public void testPipeline() throws Exception {
        var in = new ByteArrayInputStream("hello world".getBytes());
        var out = new ByteArrayOutputStream();
        var builder = StreamPipeline.builder()
            // Stage 1: uppercase
            .addHandler((input, output) -> {
                String text = new String(input.readAllBytes()).toUpperCase();
                output.write(text.getBytes());
            })
            // Stage 2: add prefix
            .addHandler((input, output) -> {
                String text = "PREFIX: " + new String(input.readAllBytes());
                output.write(text.getBytes());
            });

        try (var pipeline = builder.build()) {
            pipeline.handleRequest(in, out);
            System.out.println(new String(out.toByteArray())); // "PREFIX: HELLO WORLD"
        }

        Assertions.assertEquals("PREFIX: HELLO WORLD", new String(out.toByteArray()));
    }
}