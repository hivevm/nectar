// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.commonmark.alert;

import org.commonmark.parser.Parser;

/**
 * The AlertExtension class implements the Parser.ParserExtension interface and provides a mechanism
 * to extend the CommonMark parser by adding support for custom alert blocks.
 * <p>
 * Alert blocks are a form of custom block syntax that can be used to highlight specific note-like
 * content such as warnings, success messages, errors, or informational notes.
 * <p>
 * This implementation registers the AlertBlockParser.Factory with the parser builder, enabling the
 * parsing and handling of alert blocks within the CommonMark document.
 * <p>
 * The extension supports multi-line and single-line alert blocks, and the type of alert (e.g.,
 * NOTE, SUCCESS, WARNING, ERROR) is determined based on specific markers in the block's syntax.
 */
public record AlertExtension() implements Parser.ParserExtension {

    @Override
    public void extend(org.commonmark.parser.Parser.Builder parserBuilder) {
        parserBuilder.customBlockParserFactory(new AlertBlockParser.Factory());
    }
}
