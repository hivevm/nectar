// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util.lambda;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

/**
 * Low-level request-handling interface, Lambda stream request handlers implement AWS Lambda Function application logic
 * using input and output stream
 */
@FunctionalInterface
public interface RequestStreamHandler {
    /**
     * Handles a Lambda Function request
     *
     * @param input   The Lambda Function input stream
     * @param output  The Lambda function output stream
     * @param context The Lambda execution environment context object.
     */
    void handleRequest(InputStream input, OutputStream output, File context) throws IOException;
}