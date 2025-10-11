// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.commonmark.marker;

import org.commonmark.parser.delimiter.DelimiterProcessor;
import org.commonmark.parser.delimiter.DelimiterRun;
import org.hivevm.commonmark.marker.Marker.Decoration;

class MarkerProcessor implements DelimiterProcessor {

    private final int        length;
    private final char       character;
    private final Decoration decoration;

    /**
     * Constructs an instance of {@link MarkerProcessor}.
     */
    public MarkerProcessor() {
        this(2, '/', Decoration.Highlight);
    }

    /**
     * Constructs an instance of {@link MarkerProcessor}.
     *
     * @param length
     * @param character
     * @param decoration
     */
    protected MarkerProcessor(int length, char character, Decoration decoration) {
        this.length = length;
        this.character = character;
        this.decoration = decoration;
    }

    @Override
    public final int getMinLength() {
        return this.length;
    }

    @Override
    public final char getOpeningCharacter() {
        return this.character;
    }

    @Override
    public final char getClosingCharacter() {
        return this.character;
    }

    /*
     * @see org.commonmark.parser.delimiter.DelimiterProcessor#process(org.commonmark.
     * parser.delimiter. DelimiterRun, org.commonmark.parser.delimiter.DelimiterRun)
     */
    @Override
    public final int process(DelimiterRun openingRun, DelimiterRun closingRun) {
        if ((openingRun.length() < getMinLength()) || (closingRun.length() < getMinLength())) {
            return 0;
        }

        // Use exactly two delimiters even if we have more, and don't care about
        // internal
        // openers/closers.
        var opener = openingRun.getOpener();
        var closer = closingRun.getCloser();

        // Wrap nodes between delimiters in marker.
        var textDecoration = new Marker(this.decoration);

        var tmp = opener.getNext();
        while ((tmp != null) && (tmp != closer)) {
            var next = tmp.getNext();
            textDecoration.appendChild(tmp);
            tmp = next;
        }

        opener.insertAfter(textDecoration);
        return getMinLength();
    }
}
