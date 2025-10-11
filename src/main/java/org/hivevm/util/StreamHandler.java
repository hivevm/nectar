// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

/**
 * Represents a functional interface for handling data streams in a request-response workflow.
 * Implementations of this interface can process input data from the provided {@link InputStream}
 * and write the corresponding output to the provided {@link OutputStream}.
 * <p>
 * This interface enables stream-based processing and can be used in applications where data needs
 * to be handled in an efficient and extensible manner, such as in pipeline architectures or web
 * service handlers.
 */
@FunctionalInterface
public interface StreamHandler {

    /**
     * Processes input data from the given {@link InputStream} and writes the corresponding output
     * to the provided {@link OutputStream}.
     */
    void handleRequest(InputStream input, OutputStream output) throws IOException;
}