// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause
package org.hivevm.util;

import org.hivevm.doc.md.MarkdownReader;

import java.io.*;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public abstract class MergeReader extends Reader {

    private final Reader reader;
    private final Resolver resolver;

    private int index = 0;
    private String text = null;

    /**
     * Constructs an instance of {@link MarkdownReader}.
     *
     * @param reader
     * @param resolver
     */
    protected MergeReader(Reader reader, Resolver resolver) {
        this.reader = reader;
        this.resolver = resolver;
    }

    /**
     * Merge the files to the markdown content.
     *
     * @param title
     * @param header
     * @param reader
     * @param resolver
     * @param request
     */
    protected abstract void merge(String title, int header, Reader reader, Resolver resolver,
                                  Request request) throws IOException;

    /**
     * Reads characters into a portion of an array.
     *
     * @param buffer
     * @param offset
     * @param length
     */
    @Override
    public final int read(char[] buffer, int offset, int length) throws IOException {
        if (text == null) {
            StringWriter writer = new StringWriter();
            try (Request request = new Request(new PrintWriter(writer))) {
                merge(null, 0, reader, resolver, request);
            }
            index = 0;
            text = writer.toString();
        }

        if (index >= text.length()) {
            return -1;
        }

        if (index + length > text.length()) {
            length = text.length() - index;
        }

        text.getChars(index, index + length, buffer, offset);

        index += length;
        return length;
    }

    /**
     * Closes this stream and releases any system resources associated with it.
     */
    @Override
    public final void close() throws IOException {
        this.text = null;
    }

    /**
     * Reads all data.
     */
    public final String readAll() throws IOException {
        StringWriter writer = new StringWriter();
        try (BufferedReader reader = new BufferedReader(this)) {
            String line;
            while ((line = reader.readLine()) != null) {
                writer.write(line + "\n");
            }
        }
        return writer.toString();
    }

    /**
     * Defines a rendering request.
     */
    protected class Request implements Closeable {

        private final PrintWriter writer;
        private final List<String> indexes;
        private final Map<Integer, String> footnotes;

        /**
         * Creates an instance of {@link Request}.
         *
         * @param writer
         */
        public Request(PrintWriter writer) {
            this.writer = writer;
            this.indexes = new ArrayList<>();
            this.footnotes = new HashMap<>();
        }

        /**
         * Adds a foot node
         */
        public void addFootNote(String file, String name, String uri, String title) {
            int index = this.indexes.indexOf(String.format("%s_%s", file, name)) + 1;
            String text = String.format("[%s]: %s%s", index, uri, (title == null) ? "" : title);
            this.footnotes.put(index, text);
        }

        /**
         * Calculates the new footnote reference.
         *
         * @param file
         * @param name
         * @param text
         */
        public final String getFootNote(String file, String name, String text) {
            this.indexes.add(String.format("%s_%s", file, name));
            int index = this.indexes.size();
            return (text == null) ? String.format("[%s]", index) : String.format("[%s][%s]", text, index);
        }

        /**
         * Print the text to the writer.
         *
         * @param text
         */
        public final void write(String text) {
            this.writer.print(text);
        }

        /**
         * Print the text to the writer.
         *
         * @param text
         */
        public final void writeln(String text) {
            this.writer.println(text);
        }

        /**
         * Closes this request and releases resources associated with it.
         */
        @Override
        public final void close() throws IOException {
            if (!this.footnotes.isEmpty()) {
                this.writer.println();
                this.footnotes.keySet().stream().sorted()
                        .forEach(i -> this.writer.println(this.footnotes.get(i)));
                this.writer.println();
            }
            this.writer.close();
        }
    }
}
