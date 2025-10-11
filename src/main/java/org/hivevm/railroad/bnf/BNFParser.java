// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.bnf;

import java.io.IOException;
import java.io.Reader;
import java.io.StringReader;
import org.hivevm.railroad.bnf.BNFChunk.ChunkType;
import org.hivevm.railroad.grammar.Grammar;

public class BNFParser {

    public static Grammar parse(String string) {
        try {
            return BNFParser.parse(new StringReader(string));
        } catch (IOException e) {
            // Doesn't happen
            throw new RuntimeException(e);
        }
    }

    public static Grammar parse(Reader reader) throws IOException {
        var grammar = new Grammar();
        var builder = new StringBuilder();
        for (int x; (x = reader.read()) != -1; ) {
            char c = (char) x;
            switch (c) {
                case '=': {
                    var chunk = new BNFChunk();
                    var expressionText = loadExpression(chunk, reader, ';');
                    if (expressionText.endsWith(";"))
                        expressionText = expressionText.substring(0, expressionText.length() - 1);

                    var ruleName = builder.toString();
                    builder.delete(0, builder.length());
                    if (ruleName.endsWith(":")) {
                        ruleName = ruleName.substring(0, ruleName.length() - 1);
                        if (ruleName.endsWith(":")) {
                            ruleName = ruleName.substring(0, ruleName.length() - 1);
                        }
                    }
                    ruleName = ruleName.trim();

                    chunk.prune();
                    var expr = chunk.buildExpression().build();
                    grammar.addRule(ruleName, expr, expressionText);
                    break;
                }
                // Consider that '(' in rule name is start of a comment.
                case '(': {
                    if (reader.read() != '*')
                        throw new IllegalStateException("Expecting start of a comment after '(' but could not find '*'!");

                    char lastChar = 0;
                    for (int x2; (x2 = reader.read()) != -1; ) {
                        char c2 = (char) x2;
                        if (c2 == ')' && lastChar == '*') {
                            break;
                        }
                        lastChar = c2;
                    }
                    break;
                }
                default: {
                    if (!Character.isWhitespace(c) || !builder.isEmpty())
                        builder.append(c);
                    break;
                }
            }
        }
        return grammar;
    }

    private static String loadExpression(BNFChunk parentChunk, Reader reader, char stopChar) throws IOException {
        var builder = new StringBuilder();
        char lastChar = 0;
        var sb = new StringBuilder();
        boolean isFirst = true;
        boolean isInSpecialGroup = false;
        char specialGroupChar = 0;
        boolean isLiteral = parentChunk.getType() == ChunkType.LITERAL;
        for (int x; (x = reader.read()) != -1; ) {
            char c = (char) x;
            builder.append(c);
            if (isLiteral) {
                if (c == stopChar) {
                    String s = sb.toString();
                    parentChunk.setText(s);
                    return builder.toString();
                }
                sb.append(c);
            } else {
                if (isFirst && parentChunk.getType() == ChunkType.GROUP) {
                    switch (c) {
                        case '*':
                            isInSpecialGroup = true;
                            specialGroupChar = c;
                            break;
                        case '?':
                            isInSpecialGroup = true;
                            specialGroupChar = c;
                            break;
                    }
                }
                isFirst = false;
                if (isInSpecialGroup) {
                    if (c == ')' && lastChar == specialGroupChar) {
                        // Mutate parent group
                        switch (specialGroupChar) {
                            case '*':
                                parentChunk.setType(ChunkType.COMMENT);
                                break;
                            case '?':
                                parentChunk.setType(ChunkType.SPECIAL_SEQUENCE);
                                break;
                        }
                        String comment = sb.toString();
                        comment = comment.substring(1, comment.length() - 1).trim();
                        parentChunk.setText(comment);
                        return builder.toString();
                    }
                    if (!sb.isEmpty() || !Character.isWhitespace(c))
                        sb.append(c);
                } else {
                    if (c == stopChar) {
                        String content = sb.toString().trim();
                        if (!content.isEmpty())
                            parentChunk.addChunk(ChunkType.RULE).setText(content);
                        return builder.toString();
                    }
                    switch (c) {
                        case ',':
                        case ' ':
                        case '\n':
                        case '\r':
                        case '\t': {
                            String content = sb.toString().trim();
                            if (!content.isEmpty())
                                parentChunk.addChunk(ChunkType.RULE).setText(content);
                            sb.delete(0, sb.length());
                            //            parentChunk.addChunk(new Chunk(ChunkType.CONCATENATION));
                            break;
                        }
                        case '|': {
                            String content = sb.toString().trim();
                            if (!content.isEmpty())
                                parentChunk.addChunk(ChunkType.RULE).setText(content);
                            sb.delete(0, sb.length());
                            parentChunk.addChunk(ChunkType.ALTERNATION);
                            break;
                        }
                        case '*':
                        case '+':
                        case '?': {
                            String content = sb.toString().trim();
                            if (!content.isEmpty())
                                parentChunk.addChunk(ChunkType.RULE).setText(content);
                            sb.delete(0, sb.length());
                            parentChunk.addChunk(ChunkType.REPETITION_TOKEN).setText(String.valueOf(c));
                            break;
                        }
                        case '\"': {
                            String content = sb.toString().trim();
                            if (!content.isEmpty())
                                parentChunk.addChunk(ChunkType.RULE).setText(content);
                            sb.delete(0, sb.length());
                            var literalChunk = parentChunk.addChunk(ChunkType.LITERAL);
                            String subExpressionText = loadExpression(literalChunk, reader, '\"');
                            builder.append(subExpressionText);
                            break;
                        }
                        case '\'': {
                            String content = sb.toString().trim();
                            if (!content.isEmpty())
                                parentChunk.addChunk(ChunkType.RULE).setText(content);
                            sb.delete(0, sb.length());
                            var literalChunk = parentChunk.addChunk(ChunkType.LITERAL);
                            String subExpressionText = loadExpression(literalChunk, reader, '\'');
                            builder.append(subExpressionText);
                            break;
                        }
                        case '(': {
                            String content = sb.toString().trim();
                            if (!content.isEmpty())
                                parentChunk.addChunk(ChunkType.RULE).setText(content);
                            sb.delete(0, sb.length());
                            var groupChunk = parentChunk.addChunk(ChunkType.GROUP);
                            String subExpressionText = loadExpression(groupChunk, reader, ')');
                            builder.append(subExpressionText);
                            break;
                        }
                        case '[': {
                            String content = sb.toString().trim();
                            if (!content.isEmpty())
                                parentChunk.addChunk(ChunkType.RULE).setText(content);
                            sb.delete(0, sb.length());
                            var optionChunk = parentChunk.addChunk(ChunkType.OPTION);
                            String subExpressionText = loadExpression(optionChunk, reader, ']');
                            builder.append(subExpressionText);
                            break;
                        }
                        case '{': {
                            String content = sb.toString().trim();
                            if (!content.isEmpty())
                                parentChunk.addChunk(ChunkType.RULE).setText(content);
                            sb.delete(0, sb.length());
                            var repetitionChunk = parentChunk.addChunk(ChunkType.REPETITION);
                            repetitionChunk.setMinCount(0);
                            String subExpressionText = loadExpression(repetitionChunk, reader, '}');
                            builder.append(subExpressionText);
                            break;
                        }
                        default: {
                            if (!sb.isEmpty() || !Character.isWhitespace(c))
                                sb.append(c);
                            break;
                        }
                    }
                }
                lastChar = c;
            }
        }
        return builder.toString();
    }
}
