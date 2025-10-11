// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

public enum PageMatch {

    Blank("blank"),
    NotBlank("not-blank "),
    Odd("odd"),
    Even("even"),
    First("first"),
    Last("last"),
    Rest("rest"),
    Any("any"),
    Only("only");

    public final String Name;

    PageMatch(String name) {
        this.Name = name;
    }
}