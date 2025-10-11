// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.util;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

/**
 * Low-level request-handling interface, lambda stream request handlers implement lambda function
 * application logic using input and output stream
 */
@FunctionalInterface
public interface RequestStreamHandler<C> {

    /**
     * Handles a Lambda Function request
     *
     * @param input   The Lambda Function input stream
     * @param output  The Lambda function output stream
     * @param context The Lambda execution environment context object.
     */
    void handleRequest(InputStream input, OutputStream output, C context) throws IOException;
}
