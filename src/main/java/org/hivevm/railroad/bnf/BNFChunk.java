// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.bnf;

import java.util.ArrayList;
import java.util.List;
import org.hivevm.railroad.grammar.Choice;
import org.hivevm.railroad.grammar.Expression;
import org.hivevm.railroad.grammar.GrammarBuilder;
import org.hivevm.railroad.grammar.Repetition;
import org.hivevm.railroad.grammar.Sequence;

class BNFChunk {

    public enum ChunkType {
        RULE,
        REPETITION_TOKEN,
        //    CONCATENATION,
        ALTERNATION,
        GROUP,
        COMMENT,
        SPECIAL_SEQUENCE,
        LITERAL,
        OPTION,
        REPETITION,
        CHOICE,
    }

    private final GrammarBuilder builder;

    private ChunkType      type;
    private String         text;
    private int            minCount;
    private Integer        maxCount;
    private List<BNFChunk> chunkList;

    public BNFChunk() {
        this(ChunkType.GROUP, new GrammarBuilder());
    }

    private BNFChunk(ChunkType type, GrammarBuilder builder) {
        this.type = type;
        this.builder = builder;
    }

    private BNFChunk newInstance(ChunkType type) {
        return new BNFChunk(type, builder);
    }

    public ChunkType getType() {
        return type;
    }

    public BNFChunk setType(ChunkType type) {
        this.type = type;
        return this;
    }

    public BNFChunk setText(String text) {
        this.text = text;
        return this;
    }

    public BNFChunk setMinCount(int minCount) {
        this.minCount = minCount;
        return this;
    }

    public BNFChunk setMaxCount(Integer maxCount) {
        this.maxCount = maxCount;
        return this;
    }

    private void addChunk(BNFChunk chunk) {
        if (chunkList == null)
            chunkList = new ArrayList<>();
        chunkList.add(chunk);
    }

    private GrammarBuilder builder() {
        return builder;
    }

    public BNFChunk addChunk(ChunkType type) {
        var chunk = new BNFChunk(type, builder);
        addChunk(chunk);
        return chunk;
    }

    public final void prune() {
        boolean hasAlternation = false;
        for (int i = chunkList.size() - 1; i >= 0; i--) {
            BNFChunk chunk = chunkList.get(i);
            switch (chunk.getType()) {
                case REPETITION_TOKEN:
                    if ("*".equals(chunk.text)) {
                        chunkList.remove(i);
                        BNFChunk previousChunk = chunkList.get(i - 1);
                        Integer multiplier = null;
                        // Case of: 3 * expression
                        if (previousChunk.getType() == ChunkType.RULE) {
                            try {
                                multiplier = Integer.parseInt(previousChunk.text);
                            } catch (Exception e) {
                            }
                        }
                        if (multiplier != null) {
                            // The current one is removed, so next one is at index i.
                            var nextChunk = chunkList.get(i);
                            if (nextChunk.getType() == ChunkType.OPTION) {
                                var newChunk = newInstance(ChunkType.REPETITION);
                                newChunk.setMinCount(0);
                                newChunk.setMaxCount(multiplier);
                                for (var c : nextChunk.chunkList) {
                                    newChunk.addChunk(c);
                                }
                                chunkList.remove(i);
                                chunkList.set(i - 1, newChunk);
                            }
                            else {
                                var newChunk = newInstance(ChunkType.REPETITION);
                                newChunk.setMinCount(multiplier);
                                newChunk.setMaxCount(multiplier);
                                newChunk.addChunk(nextChunk);
                                chunkList.remove(i);
                                chunkList.set(i - 1, newChunk);
                            }
                        }
                        else {
                            var newChunk = newInstance(ChunkType.REPETITION);
                            newChunk.setMinCount(0);
                            newChunk.addChunk(previousChunk);
                            chunkList.set(i - 1, newChunk);
                        }
                    }
                    else if ("+".equals(chunk.text)) {
                        chunkList.remove(i);
                        var newChunk = newInstance(ChunkType.REPETITION);
                        newChunk.setMinCount(1);
                        var previousChunk = chunkList.get(i - 1);
                        newChunk.addChunk(previousChunk);
                        chunkList.set(i - 1, newChunk);
                    }
                    else if ("?".equals(chunk.text)) {
                        chunkList.remove(i);
                        var newChunk = newInstance(ChunkType.OPTION);
                        var previousChunk = chunkList.get(i - 1);
                        newChunk.addChunk(previousChunk);
                        chunkList.set(i - 1, newChunk);
                    }
                    break;
                case COMMENT:
                    // For now, nothing to do
                    chunkList.remove(i);
                case ALTERNATION:
                    hasAlternation = true;
                    break;
                case GROUP:
                    // Group could be empty
                    if (chunk.chunkList != null) {
                        chunk.prune();
                        if (chunk.chunkList.size() == 1)
                            chunkList.set(i, chunk.chunkList.getFirst());
                    }
                    break;
                case OPTION:
                case REPETITION:
                    chunk.prune();
                    break;
            }
        }
        if (hasAlternation) {
            List<List<BNFChunk>> alternationSequenceList = new ArrayList<>();
            alternationSequenceList.add(new ArrayList<>());
            for (var chunk : chunkList) {
                if (chunk.getType() == ChunkType.ALTERNATION)
                    alternationSequenceList.add(new ArrayList<>());
                else
                    alternationSequenceList.getLast().add(chunk);
            }
            var choiceChunk = newInstance(ChunkType.CHOICE);
            for (List<BNFChunk> subList : alternationSequenceList) {
                if (subList.size() == 1)
                    choiceChunk.addChunk(subList.getFirst());
                else {
                    var groupChunk = newInstance(ChunkType.GROUP);
                    for (var c : subList) {
                        groupChunk.addChunk(c);
                    }
                    choiceChunk.addChunk(groupChunk);
                }
            }
            chunkList.clear();
            chunkList.add(choiceChunk);
        }
    }

    public final GrammarBuilder buildExpression() {
        return switch (type) {
            case GROUP -> {
                if (chunkList == null) // Group is empty.
                    yield builder().sequence(0);

                if (chunkList.size() == 1)
                    yield chunkList.getFirst().buildExpression();

                for (var chunk : chunkList) {
                    chunk.buildExpression();
                }
                yield builder().sequence(chunkList.size());
            }
            case CHOICE -> {
                if (chunkList.size() == 1)
                    yield chunkList.getFirst().buildExpression();

                int exprCount = 0;
                boolean hasLine = false;
                for (var chunk : chunkList) {
                    Expression expression = chunk.buildExpression().build();
                    if (expression instanceof Repetition(
                        Expression expression1, int minRepetitionCount, Integer maxRepetitionCount
                    )) {
                        if (minRepetitionCount == 0) {
                            if (maxRepetitionCount == null || maxRepetitionCount != 1)
                                expression = new Repetition(expression1, 1, maxRepetitionCount);
                            else
                                expression = expression1;
                            hasLine = true;
                        }
                    }
                    if (expression instanceof Choice(List<Expression> expressions)) {
                        exprCount += expressions.size();
                        expressions.forEach(builder::add);
                    }
                    else {
                        exprCount++;
                        builder().add(expression);
                    }
                }
                if (hasLine && !isNoop(builder().current())) {
                    exprCount++;
                    builder().sequence(0);
                }
                yield builder().choice(exprCount);
            }
            case RULE -> builder().ruleReference(text);
            case LITERAL -> builder().literal(text);
            case SPECIAL_SEQUENCE -> builder().specialSequence(text);
            case OPTION -> {
                if (chunkList.size() == 1) {
                    var subChunk = chunkList.getFirst();
                    if (subChunk.getType() == ChunkType.CHOICE) {
                        var newChunk = newInstance(ChunkType.CHOICE);
                        for (var cChunk : subChunk.chunkList) {
                            newChunk.addChunk(cChunk);
                        }
                        newChunk.addChunk(newInstance(ChunkType.GROUP));
                        yield newChunk.buildExpression();
                    }
                    yield subChunk.buildExpression().repetition(0, 1);
                }

                chunkList.forEach(BNFChunk::buildExpression);
                yield builder().sequence(chunkList.size()).repetition(0, 1);
            }
            case REPETITION -> {
                if (chunkList.size() == 1)
                    yield chunkList.getFirst().buildExpression().repetition(minCount, maxCount);

                chunkList.forEach(BNFChunk::buildExpression);
                yield builder().sequence(chunkList.size()).repetition(minCount, maxCount);
            }
            default -> throw new IllegalStateException("Type should not be reachable: " + type);
        };
    }

    private static boolean isNoop(Expression expression) {
        return expression == null
            || expression instanceof Sequence && ((Sequence) expression).expressions().isEmpty();
    }
}
