// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.commonmark.table;

import org.commonmark.parser.Parser;

/**
 * A record that implements {@link Parser.ParserExtension} to extend the functionality of a
 * {@link Parser} by adding support for custom table parsing. This class is responsible for
 * registering a custom block parser factory for table parsing when the parser is being built.
 * <p>
 * The registered {@link TableParser.Factory} enables the parser to recognize and process table
 * blocks in the parsed content. It parses table headers, separators, and rows based on
 * table-specific syntax.
 * <p>
 * The table parser supports features such as: - Parsing table headers and rows. - Recognizing
 * alignment and column widths specified in the table. - Processing inline content within table
 * cells.
 */
public record TableExtension() implements Parser.ParserExtension {

    @Override
    public void extend(Parser.Builder parserBuilder) {
        parserBuilder.customBlockParserFactory(new TableParser.Factory());
    }
}
