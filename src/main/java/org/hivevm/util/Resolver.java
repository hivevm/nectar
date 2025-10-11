// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause
package org.hivevm.util;

import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Path;

public interface Resolver {

    String getPath(String path);

    boolean exists(String path);

    Resolver getResolver(String path);

    InputStream getInputStream(String path) throws IOException;

    class PathResolver implements Resolver {

        private final Path root;
        private final Path base;

        public PathResolver(Path root, Path base) {
            this.root = root;
            this.base = base;
        }

        public PathResolver(File workingDir) {
            this(workingDir.toPath(), workingDir.toPath());
        }

        public final String getPath(String path) {
            Path newPath = this.base.resolve(path);
            return this.root.relativize(newPath).toString();
        }

        public final boolean exists(String path) {
            return this.base.resolve(path).toFile().exists();
        }

        public final Resolver getResolver(String path) {
            return new PathResolver(this.root, this.base.resolve(path).toFile().getParentFile().toPath());
        }

        public final InputStream getInputStream(String path) throws IOException {
            return new FileInputStream(this.base.resolve(path).toFile());
        }
    }
}
