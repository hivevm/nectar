// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

import java.util.Map;

/**
 * The {@link FoUtil} is an utility to build {@link FoNode}.
 */
class FoUtil {

    private static void writeAttr(StringBuffer buffer, String name, String value) {
        if (value != null) {
            buffer.append(" ");
            buffer.append(name);
            buffer.append("=\"");
            buffer.append(value);
            buffer.append("\"");
        }
    }

    private static void writeAttrs(StringBuffer buffer, Map<String, String> attrs) {
        for (Map.Entry<String, String> e : attrs.entrySet()) {
            FoUtil.writeAttr(buffer, e.getKey(), e.getValue());
        }
    }

    static void writeStart(StringBuffer buffer, String name, Map<String, String> attrs) {
        buffer.append("<");
        buffer.append(name);
        FoUtil.writeAttrs(buffer, attrs);
        buffer.append(">");
    }

    static void writeEnd(StringBuffer buffer, String name) {
        buffer.append("</");
        buffer.append(name);
        buffer.append(">");
    }

    static void writeEmpty(StringBuffer buffer, String name, Map<String, String> attrs) {
        buffer.append("<");
        buffer.append(name);
        FoUtil.writeAttrs(buffer, attrs);
        buffer.append("/>");
    }
}
