// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.md;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.Reader;
import java.util.UUID;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Collectors;
import org.hivevm.util.MergeReader;
import org.hivevm.util.Replacer;
import org.hivevm.util.Resolver;


/**
 * The {@link MarkdownReader} implements a reader based on MARKDOWN. The Reader starts from an
 * {@link InputStream} an includes referenced files to create a single huge MARKDOWN file.
 */
public class MarkdownReader extends MergeReader {

    // File includes: [TITLE?](RELATIVE_PATH)
    private static final Pattern INCLUDE =
            Pattern.compile("^(?:([#]+)\\s*)?\\[([^\\]]*)\\]\\((.+\\.md)\\)", Pattern.CASE_INSENSITIVE);

    // Header definition: #+ HEADER
    private static final Pattern HEADER = Pattern.compile("^([#]+)\\s*(.+)",
            Pattern.CASE_INSENSITIVE);

    // Footnote definition: [ID]: URL "TITLE"
    private static final Pattern FOOTNOTE =
            Pattern.compile("^\\[(\\w+)\\]:\\s([^\\s]+)(\\s\".+\")?", Pattern.CASE_INSENSITIVE);

    // Footnote reference: [TEXT]?[ID]
    private static final Pattern FOOTNOTE_REF =
            Pattern.compile("(?:\\[([^\\]]*)\\])\\[(\\w+)\\]", Pattern.CASE_INSENSITIVE);

    // Image definition: ![](IMAGE_PATH)
    private static final Pattern IMAGE = Pattern.compile("!\\[([^\\]]*)\\]\\(([^\\)]+)\\)",
            Pattern.CASE_INSENSITIVE);

    /**
     * Constructs an instance of {@link MarkdownReader}.
     *
     * @param reader
     * @param resolver
     */
    public MarkdownReader(Reader reader, Resolver resolver) {
        super(reader, resolver);
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
    @Override
    protected final void merge(String title, int header, Reader reader, Resolver resolver, Request request)
            throws IOException {
        boolean isCode = false;
        String lastLine = "";
        int level = header;
        String name = UUID.randomUUID().toString();
        try (BufferedReader buffered = new BufferedReader(reader)) {
            for (String line : buffered.lines().collect(Collectors.toList())) {
                if (line.startsWith("~~~")) {
                    request.writeln(line);
                    isCode = !isCode;
                    continue;
                }

                if (isCode) {
                    request.writeln(line);
                    continue;
                }

                // Ignore lines that starting with a tab
                if (line.startsWith("\t")) {
                    request.writeln(line);
                    continue;
                }

                // Avoid not rendered tables
                if (line.trim().startsWith("|")) {
                    if (!lastLine.startsWith("|") && !lastLine.isEmpty())
                        request.writeln("\n");
                }

                lastLine = line.trim();

                // Processes file includes.
                Matcher matcher = MarkdownReader.INCLUDE.matcher(line);
                if (matcher.find()) {
                    String path = matcher.group(3);
                    if (resolver.exists(path)) {
                        int hash = matcher.group(1) == null ? level : Integer.parseInt(matcher.group(1)) - 1;
                        String subtitle = matcher.group(2) == null ? title : matcher.group(2);
                        try (Reader r = new InputStreamReader(resolver.getInputStream(path))) {
                            request.writeln("");
                            merge(subtitle, hash, r, resolver.getResolver(path), request);
                        }
                    }
                    continue;
                }

                // Update headers
                matcher = MarkdownReader.HEADER.matcher(line);
                if (matcher.find()) {
                    level = header + matcher.group(1).length();
                    String prefix = "#############".substring(0, level);
                    if (matcher.group(1).length() == 1 && (title != null)) {
                        request.writeln(prefix + " " + title);
                    } else {
                        request.writeln(prefix + " " + matcher.group(2));
                    }
                    continue;
                }

                // Update footnotes
                matcher = MarkdownReader.FOOTNOTE.matcher(line);
                if (matcher.find()) {
                    request.addFootNote(name, matcher.group(1), matcher.group(2), matcher.group(3));
                    continue;
                }

                // Replace all link references
                line = Replacer.replaceAll(line, MarkdownReader.FOOTNOTE_REF,
                        m -> request.getFootNote(name, m.group(2), m.group(1)));

                // Normalize image paths
                line = Replacer.replaceAll(line, MarkdownReader.IMAGE,
                        m -> m.group(2).startsWith("/") || m.group(2).startsWith("data:") ? m.group(0)
                                : String.format("![%s](%s)", m.group(1), resolver.getPath(m.group(2))));

                request.writeln(line);
            }
        }
    }
}
