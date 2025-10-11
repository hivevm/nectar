// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util;

import java.io.Closeable;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.PipedInputStream;
import java.io.PipedOutputStream;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/// Represents a pipeline for stream-based processing where each stage of the pipeline processes
/// input data and passes its output to the next stage. Each stage is defined by a
/// {@link StreamHandler}.
///
/// The {@code StreamPipeline} leverages multithreading to execute each stage of the pipeline
/// concurrently. It ensures that data flow is properly managed between stages via piped streams.
///
/// Features:
/// - Sequential and concurrent execution of handlers in the pipeline.
/// - Customizable pipeline stages via the {@link StreamPipeline.Builder}.
/// - Resource management through automatic thread cleanup.
///
/// Thread Safety: This class is thread-safe as it ensures thread isolation when executing handlers
/// in the pipeline.
///
/// Resource Management: The {@code StreamPipeline} manages an internal {@link ExecutorService} to
/// execute pipeline stages in separate threads. Users must invoke the {@link #close()} method after
/// the pipeline is no longer needed to release resources and shut down the internal thread pool.
public class StreamPipeline implements StreamHandler, Closeable {

    private final List<StreamHandler> handlers;
    private final ExecutorService     executor;

    private StreamPipeline(List<StreamHandler> handlers) {
        this.handlers = handlers;
        this.executor = Executors.newVirtualThreadPerTaskExecutor();
    }

    /// Executes the pipeline of handlers provided in the StreamPipeline, processing the input
    /// stream through each handler and directing the output to the next stage or the final output
    /// stream. Each handler is executed in a separate thread to enable parallel processing.
    public final void handleRequest(InputStream input, OutputStream output) throws IOException {
        var current = input;

        for (int i = 0; i < handlers.size(); i++) {
            if (i == handlers.size() - 1) // Last stage - write to final output
                handlers.get(i).handleRequest(current, output);
            else { // Create pipe for next stage
                var pipeOut = new PipedOutputStream();
                var pipeIn = new PipedInputStream(pipeOut);

                var stageInput = current;
                var handler = handlers.get(i);

                // Run stage in separate thread
                executor.submit(() -> {
                    try {
                        handler.handleRequest(stageInput, pipeOut);
                        pipeOut.close();
                    } catch (IOException e) {
                        throw new RuntimeException(e);
                    }
                });

                current = pipeIn;
            }
        }
    }

    /// Shuts down the internal {@link ExecutorService} used by the pipeline.
    ///
    /// This method ensures that all threads in the internal thread pool are terminated. Once the
    /// {@code close()} method is invoked, the pipeline cannot be executed further. It is
    /// recommended to call this method when the {@link StreamPipeline} is no longer needed to allow
    /// proper cleanup of system resources.
    public void close() {
        executor.shutdown();
    }

    public static Builder builder() {
        return new Builder();
    }

    /// A builder class used to construct instances of {@link StreamPipeline}.
    ///
    /// The {@code Builder} class provides a mechanism to incrementally add handlers to the pipeline
    /// and create a fully configured {@link StreamPipeline} instance.
    public static class Builder {

        private final List<StreamHandler> handlers;

        public Builder() {
            this.handlers = new ArrayList<>();
        }

        public Builder addHandler(StreamHandler handler) {
            handlers.add(handler);
            return this;
        }

        public StreamPipeline build() {
            return new StreamPipeline(handlers);
        }
    }
}