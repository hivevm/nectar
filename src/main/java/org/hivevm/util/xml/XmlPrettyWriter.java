// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util.xml;

import java.lang.reflect.InvocationHandler;
import java.lang.reflect.Method;
import java.util.HashMap;
import java.util.Map;
import javax.xml.stream.XMLStreamWriter;
import org.jspecify.annotations.NonNull;

/**
 * The {@link XmlPrettyWriter} implements an XML formatter, that can be invoked by a proxy.
 */
final class XmlPrettyWriter implements InvocationHandler {

    private static final String INDENT_CHAR   = " ";
    private static final String LINEFEED_CHAR = "\n";


    private final int             indent;
    private final XMLStreamWriter writer;


    private       int                   depth          = 0;
    private final Map<Integer, Boolean> hasTextElement = new HashMap<>();

    /**
     * Constructs an instance of {@link XmlPrettyWriter}.
     */
    public XmlPrettyWriter(@NonNull XMLStreamWriter writer, int indent) {
        this.writer = writer;
        this.indent = indent;
    }

    /**
     * Needs to be BEFORE the actual event, so that for instance the sequence writeStartElem,
     * writeAttr, writeStartElem, writeEndElem, writeEndElem is correctly handled
     */
    @Override
    public Object invoke(Object proxy, Method method, Object[] args) throws Throwable {
        switch (method.getName()) {
            case "writeStartElement":
                // update state of parent node
                if (this.depth > 0) {
                    this.hasTextElement.put(this.depth - 1, false);
                }

                // reset state of current node
                this.hasTextElement.put(this.depth, false);
                // indent for current depth
                this.writer.writeCharacters(XmlPrettyWriter.LINEFEED_CHAR);
                this.writer.writeCharacters(indent(this.depth * this.indent));
                this.depth++;
                break;

            case "writeEndElement":
                this.depth--;
                if (!this.hasTextElement.get(this.depth)) {
                    this.writer.writeCharacters(XmlPrettyWriter.LINEFEED_CHAR);
                    this.writer.writeCharacters(indent(this.depth * this.indent));
                }
                break;

            case "writeEmptyElement":
            case "writeComment":
                // indent for current depth
                this.writer.writeCharacters(XmlPrettyWriter.LINEFEED_CHAR);
                this.writer.writeCharacters(indent(this.depth * this.indent));
                break;

            case "writeCData":
            case "writeCharacters":
                // update state of parent node
                if (this.depth > 0) {
                    this.hasTextElement.put(this.depth - 1, true);
                }
                break;
        }
        method.invoke(this.writer, args);
        return null;
    }

    private String indent(int d) {
        String text = "";
        while (d-- > 0) {
            text += XmlPrettyWriter.INDENT_CHAR;
        }
        return text;
    }
}