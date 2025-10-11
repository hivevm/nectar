// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.commonmark.alert;

import org.commonmark.node.Block;
import org.commonmark.parser.block.AbstractBlockParser;
import org.commonmark.parser.block.AbstractBlockParserFactory;
import org.commonmark.parser.block.BlockContinue;
import org.commonmark.parser.block.BlockStart;
import org.commonmark.parser.block.MatchedBlockParser;
import org.commonmark.parser.block.ParserState;

/**
 * The AlertBlockParser class extends AbstractBlockParser and is responsible for parsing custom
 * alert blocks within a CommonMark document. An alert block includes specific types of content such
 * as informational notes, success messages, warnings, and errors.
 * <p>
 * AlertBlockParser supports both single-line and multi-line alert blocks, determined by specific
 * markers in the block's syntax. The parsed content is encapsulated in an AlertBlock object, which
 * contains the corresponding alert type.
 */
public class AlertBlockParser extends AbstractBlockParser {

    private final AlertBlock block;
    private       boolean    multiline;

    public AlertBlockParser(Alert type, boolean multiline) {
        this.block = new AlertBlock(type);
        this.multiline = multiline;
    }

    @Override
    public boolean isContainer() {
        return true;
    }

    @Override
    public boolean canContain(Block block) {
        return (block != null) && !AlertBlock.class.isAssignableFrom(block.getClass());
    }

    @Override
    public AlertBlock getBlock() {
        return this.block;
    }

    @Override
    public BlockContinue tryContinue(ParserState state) {
        var content = state.getLine().getContent();
        var line = content.subSequence(state.getColumn() + state.getIndent(),
            content.length());

        if (!this.multiline) {
            return BlockContinue.finished();
        }

        if ((line.length() > 1) && (line.charAt(0) == '!') && (line.charAt(1) == '!')) {
            this.multiline = false;
            return BlockContinue.finished();
        }

        return BlockContinue.atColumn(state.getColumn() + state.getIndent());
    }

    /**
     * The {@link Factory} class to create an {@link AlertBlockParser}.
     */
    public static class Factory extends AbstractBlockParserFactory {

        @Override
        public BlockStart tryStart(ParserState state, MatchedBlockParser matchedBlockParser) {
            var content = state.getLine().getContent();
            var line = content.subSequence(state.getColumn(), content.length());

            var matcher = Alert.matcher(line);
            if (matcher.matches()) {
                var isMultiline = "!".equals(matcher.group(2));
                var parser = new AlertBlockParser(Alert.of(matcher.group(3)), isMultiline);
                return BlockStart.of(parser)
                    .atColumn(state.getColumn() + state.getIndent() + matcher.end(1));
            }
            return BlockStart.none();
        }
    }
}
