// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo;


/**
 * The Fo interface is a collection of constants and methods that are central to FO (Formatting
 * Objects) generation and manipulation within the document processing pipeline.
 * <p>
 * This interface includes commonly used page set constants as well as utility methods for encoding
 * text to prevent conflicts with XML reserved characters.
 */
public interface Fo {

    String BOOK_ID          = "ROOT";
    String PAGESET_BOOK     = "book";
    String PAGESET_CHAPTER  = "chapter";
    String PAGESET_STANDARD = "standard";

    /**
     * Encodes the specified text by replacing XML-reserved characters with their corresponding
     * entity references to ensure proper XML formatting. The method replaces '&' with "&amp;", '<'
     * with "&lt;", and '>' with "&gt;".
     */
    static String encode(String text) {
        return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
    }
}
