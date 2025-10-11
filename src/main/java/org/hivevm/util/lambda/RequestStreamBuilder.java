// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util.lambda;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/**
 * A Builder to connect different {@link RequestStreamHandler} in a pipeline.
 */
public class RequestStreamBuilder implements Iterable<RequestStreamHandler> {

    private final List<RequestStreamHandler> handlers;

    public RequestStreamBuilder() {
        this.handlers = new ArrayList<>();
    }

    /**
     * Appends a {@link RequestStreamHandler} builder.
     *
     * @param handler
     */
    public RequestStreamBuilder append(RequestStreamHandler handler) {
        this.handlers.add(handler);
        return this;
    }

    @Override
    public Iterator<RequestStreamHandler> iterator() {
        return handlers.iterator();
    }

    /**
     * Creates a composing {@link RequestStreamHandler}.
     */
    public RequestStreamHandler build() {
        return (in, out, ctx) -> {
            ByteArrayOutputStream bytes = null;
            for (RequestStreamHandler h : handlers) {
                bytes = new ByteArrayOutputStream();
                h.handleRequest(in, bytes, ctx);

                in.close();
                in = new ByteArrayInputStream(bytes.toByteArray());
                bytes.close();
            }
            out.write(bytes.toByteArray());
        };
    }
}