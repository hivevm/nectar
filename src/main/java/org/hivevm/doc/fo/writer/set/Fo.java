// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer.set;

/**
 * The {@link Fo} is an interface for Apache Formating Objects.
 */
public interface Fo {

    /**
     * Set an attribute.
     */
    Fo set(String name, String value);
}
