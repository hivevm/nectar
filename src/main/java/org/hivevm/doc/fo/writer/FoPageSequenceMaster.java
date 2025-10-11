// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

import org.hivevm.doc.template.PageMatch;

/**
 * The {@link FoPageSequenceMaster} class.
 */
public class FoPageSequenceMaster extends FoNode {

    private final FoNode master;

    public enum BlankOrNot {

        Blank("blank"),
        NotBlank("not-blank "),
        Any("any");

        public final String value;

        BlankOrNot(String value) {
            this.value = value;
        }
    }

    public enum OddOrEven {

        Odd("odd"),
        Even("even"),
        Any("any");

        public final String value;

        OddOrEven(String value) {
            this.value = value;
        }
    }

    public enum Position {

        First("first"),
        Last("last"),
        Rest("rest"),
        Any("any"),
        Only("only");

        public final String value;

        Position(String value) {
            this.value = value;
        }
    }

    /**
     * Constructs an instance of {@link FoPageSequenceMaster}.
     *
     * @param name
     * @param layout
     */
    FoPageSequenceMaster(String name, FoNode layout) {
        super("page-sequence-master", layout);
        set("master-name", name);
        this.master = new FoNode("repeatable-page-master-alternatives", this);
    }

    public FoNode addPage(String name, PageMatch match) {
        FoNode node = new FoNode("conditional-page-master-reference", this.master);
        node.set("master-reference", name);
        switch (match) {
            case Odd -> node.set("odd-or-even", OddOrEven.Odd.value);
            case Even -> node.set("odd-or-even", OddOrEven.Even.value);
            case Rest -> node.set("page-position", Position.Rest.value);
            case First -> node.set("page-position", Position.First.value);
            case Last -> node.set("page-position", Position.Last.value);
            case Only -> node.set("page-position", Position.Only.value);
            case Blank -> node.set("blank-or-not-blank", BlankOrNot.Blank.value);
            case NotBlank -> node.set("blank-or-not-blank", BlankOrNot.NotBlank.value);
        }
        return this;
    }
}
