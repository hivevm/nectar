// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

import java.io.File;
import java.util.Map;

/**
 *
 */
public record TestContext(File workingDir, Map<String, String> environment, File target) {

    private static TestContext LOCAL;
    private static TestContext DEFAULTS;

    public static TestContext local() {
        if (LOCAL == null) {
            var env = Map.of("VERSION", "25.04");
            var workingDir = new File(".").getAbsoluteFile();
            var target = new File(System.getProperty("java.io.tmpdir"));
            LOCAL = new TestContext(workingDir, env, target);
        }
        return LOCAL;
    }

    public static TestContext defaults() {
        if (DEFAULTS == null) {
            var env = Map.of("VERSION", "25.04");
            var workingDir = new File("/data/hivevm/hivevm");
            var target = new File(System.getProperty("java.io.tmpdir"));
            DEFAULTS = new TestContext(workingDir, env, target);
        }
        return DEFAULTS;
    }
}
