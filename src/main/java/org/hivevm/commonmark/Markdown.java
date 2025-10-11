// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.commonmark;

import java.io.File;
import java.io.FileReader;
import java.io.IOException;
import java.io.Reader;
import java.util.List;
import org.commonmark.ext.task.list.items.TaskListItemsExtension;
import org.commonmark.node.Node;
import org.commonmark.parser.Parser;
import org.hivevm.commonmark.alert.AlertExtension;
import org.hivevm.commonmark.image.ImageExtension;
import org.hivevm.commonmark.marker.MarkerExtension;
import org.hivevm.commonmark.table.TableExtension;

/**
 * An interface for handling Markdown parsing and configuration using the CommonMark library with
 * additional custom extensions to support enhanced Markdown features.
 */
public interface Markdown {

    /**
     * Creates and returns a CommonMark {@link Parser} instance configured with default extensions.
     * The parser is extended with the following capabilities: - ImageExtension for custom image
     * attributes. - TableExtension for custom table parsing. - AlertExtension for custom alert
     * blocks tailored for highlighting notes, warnings, or informational content. - MarkerExtension
     * for GitHub Flavored Markdown strikethrough and underline support. - TaskListItemsExtension
     * for handling task list items within Markdown.
     * <p>
     * This method centralizes the configuration of the parser with a predefined set of extensions
     * that enhance the Markdown parsing capabilities.
     */
    static Parser parser() {
        var extensions = List.of(new ImageExtension(),
            new TableExtension(),
            new AlertExtension(),
            new MarkerExtension(),
            TaskListItemsExtension.create()
        );
        return Parser.builder().extensions(extensions).build();
    }

    /**
     * Parses a given Markdown input string and converts it into a CommonMark {@link Node}. Uses the
     * default parser configuration with predefined extensions to support additional Markdown
     * features.
     */
    static Node parse(String input) {
        return parser().parse(input);
    }

    /**
     * Parses the input provided through a {@link Reader} containing Markdown content and converts
     * it into a CommonMark {@link Node}. The method uses the default parser configuration with
     * extensions to support additional features such as images, tables, alerts, markers, and task
     * list items.
     */
    static Node parse(Reader input) throws IOException {
        return parser().parseReader(input);
    }

    /**
     * Parses the Markdown content from a given file and converts it into a CommonMark {@link Node}.
     * This method uses a parser configured with default extensions to handle additional features
     * such as tables, images, alerts, and task lists.
     */
    static Node parse(File input) throws IOException {
        try (Reader reader = new FileReader(input)) {
            return parser().parseReader(reader);
        }
    }
}
