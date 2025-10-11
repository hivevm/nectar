// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer.set;

/**
 * The {@link FoIndent} class.
 */
public interface FoIndent<F extends FoIndent<?>> extends Fo {

    @SuppressWarnings("unchecked")
    default F setStartIndent(String value) {
        set("start-indent", value);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setEndIndent(String align) {
        set("end-indent", align);
        return (F) this;
    }

    @SuppressWarnings("unchecked")
    default F setEndIndentLastLine(String align) {
        set("last-line-end-indent", align);
        return (F) this;
    }
}
