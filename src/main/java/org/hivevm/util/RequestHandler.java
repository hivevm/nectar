// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.util;

/**
 * Request handlers implement lambda function application logic using plain old java objects as
 * input and output.
 *
 * @param <I> The input parameter type
 * @param <O> The output parameter type
 */
@FunctionalInterface
public interface RequestHandler<O, I, C> {

    /**
     * Handles a Lambda Function request
     *
     * @param input   The Lambda Function input
     * @param context The Lambda execution environment context object.
     * @return The Lambda Function output
     */
    O handleRequest(I input, C context);
}
