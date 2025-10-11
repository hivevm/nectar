// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util.xml;

import java.io.Closeable;
import java.math.BigDecimal;
import java.text.NumberFormat;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Map;
import java.util.Stack;
import javax.xml.stream.XMLStreamException;
import javax.xml.stream.XMLStreamWriter;
import javax.xml.transform.Transformer;

/**
 * The {@link XmlBuilder} implements an XML based builder. The {@link XmlBuilder} is able to create
 * pretty XML content on the {@link XMLStreamWriter} without a {@link Transformer}.
 */
public class XmlBuilder
    implements Closeable {

    private final XMLStreamWriter writer;

    private boolean deferred;
    private String  namespace;


    private final Stack<String>       nodes      = new Stack<>();
    private final Map<String, String> namespaces = new HashMap<>();

    private final Map<String, String> attributeNs = new HashMap<>();
    private final Map<String, String> attribute   = new LinkedHashMap<>();

    /**
     * Creates a new named {@link XmlBuilder} depending optional on another instance.
     */
    public XmlBuilder(XMLStreamWriter writer) throws XMLStreamException {
        this.writer = writer;
        this.writer.writeStartDocument("UTF-8", "1.0");
    }

    /**
     * Get the {@link XMLStreamWriter}.
     */
    protected final XMLStreamWriter getWriter() {
        return this.writer;
    }

    /**
     * Set the current namespace.
     */
    public final void setNamespace(String namespace) {
        this.namespace = namespace;
    }

    /**
     * Add the namespace prefixes.
     */
    public final void addNamespace(String prefix, String namespace) {
        this.namespaces.put(namespace, prefix);
    }

    /**
     * Add a XML document attribute.
     */
    public final void setAttribute(String name, String value, String namespace) {
        this.attribute.put(name, value);
        if (namespace != null) {
            this.attributeNs.put(name, namespace);
        }
    }

    /**
     * Writes a string value for the named attribute.
     */
    public final XmlBuilder set(String name, String value) {
        return set(name, value, false);
    }

    /**
     * Writes a boolean value for the named attribute.
     */
    public final XmlBuilder set(String name, boolean value) {
        return set(name, value ? Boolean.TRUE.toString().toLowerCase() : null);
    }

    /**
     * Writes a numeric value for the named attribute.
     */
    public final XmlBuilder set(String name, double value) {
        boolean isDefault = Double.isNaN(value) || (value == 0.0);
        return set(name,
            isDefault ? null : XmlBuilder.getBigDecimalValue(BigDecimal.valueOf(value)));
    }

    /**
     * Writes an integer value for the named attribute.
     */
    public XmlBuilder set(String name, int value) {
        return set(name, value == 0 ? null : Integer.toString(value));
    }

    /**
     * Writes an enumeration value for the named attribute.
     */
    public final XmlBuilder set(String name, Enum<?> value) {
        return set(name,
            ((value == null) || (value.ordinal() == 0)) ? null : value.name().toUpperCase());
    }

    /**
     * Writes a string value for the named attribute.
     */
    private XmlBuilder set(String name, String value, boolean empty) {
        if ((value != null) && (empty || !value.isEmpty())) {
            this.attribute.put(name, value);
        }
        return this;
    }

    /**
     * Writes a comment for the next element.
     */
    public XmlBuilder addComment(String comment) {
        if (this.deferred) {
            startElement(nodes.peek(), false);
        }

        try {
            getWriter().writeComment(comment);
        } catch (XMLStreamException e) {
            throw new IllegalArgumentException("Failed to write comment", e);
        }
        return this;
    }

    /**
     * Writes a text value.
     */
    public final XmlBuilder addContentText(String text) {
        return addContent(text, false);
    }

    /**
     * Writes a text value in CData.
     */
    public final XmlBuilder addContentData(String data) {
        return addContent(data, true);
    }

    /**
     * Writes a data value.
     */
    private XmlBuilder addContent(String value, boolean isCData) {
        if ((value != null) && !value.isEmpty()) {
            if (this.deferred) {
                startElement(nodes.peek(), false);
            }

            try {
                if (isCData) {
                    getWriter().writeCData(value);
                }
                else {
                    getWriter().writeCharacters(value);
                }
            } catch (XMLStreamException e) {
                throw new IllegalArgumentException(String.format("Failed to generate %s", value),
                    e);
            }
        }
        return this;
    }

    /**
     * Append a node on the top of the {@link XmlBuilder}.
     */
    public final XmlBuilder push(String name) {
        if (this.deferred) {
            startElement(this.nodes.peek(), false);
        }
        this.deferred = true;
        this.nodes.push(name);
        return this;
    }

    /**
     * Start the element.
     */
    private void startElement(String name, boolean isEmpty) {
        try {
            if (isEmpty) {
                XmlBuilder.writeEmptyElement(getWriter(), name, this.namespace);
            }
            else {
                XmlBuilder.writeElement(getWriter(), name, this.namespace);
            }

            if (this.nodes.size() == 1) {
                for (String ns : this.namespaces.keySet()) {
                    getWriter().writeNamespace(this.namespaces.get(ns), ns);
                }
            }

            for (String key : this.attribute.keySet()) {
                try {
                    XmlBuilder.writeAttribute(getWriter(), key, this.attribute.get(key),
                        this.attributeNs.get(key));
                } catch (XMLStreamException e) {
                    throw new IllegalArgumentException(
                        String.format("Failed to generate attribute %s=%s", key,
                            this.attribute.get(key)), e);
                }
            }

            this.attribute.clear();
            this.deferred = false;
        } catch (XMLStreamException e) {
            throw new IllegalArgumentException(String.format("Failed to generate %s", name), e);
        }
    }

    /**
     * Build the top most node from {@link XmlBuilder}.
     */
    public final String build() {
        if (this.deferred) {
            startElement(nodes.peek(), true);
        }
        else {
            try {
                getWriter().writeEndElement();
            } catch (XMLStreamException e) {
                throw new IllegalArgumentException("Failed to generate", e);
            }
        }
        return this.nodes.pop();
    }

    /**
     * Closes this resource, relinquishing any underlying resources. This method is invoked
     * automatically on objects managed by the {@code try}-with-resources statement.
     */
    @Override
    public void close() {
        try {
            if (this.deferred) {
                startElement(nodes.peek(), true);
            }

            getWriter().writeEndDocument();
            getWriter().flush();
            getWriter().close();
        } catch (XMLStreamException e) {
            throw new IllegalArgumentException("Failed to close document", e);
        }
    }

    /**
     * Write an XML element with an optional namespace.
     */
    private static void writeElement(XMLStreamWriter writer, String name, String namespace)
        throws XMLStreamException {
        if (namespace == null) {
            writer.writeStartElement(name);
        }
        else {
            writer.writeStartElement(namespace, name);
        }
    }

    /**
     * Write an empty XML element with an optional namespace.
     */
    private static void writeEmptyElement(XMLStreamWriter writer, String name, String namespace)
        throws XMLStreamException {
        if (namespace == null) {
            writer.writeEmptyElement(name);
        }
        else {
            writer.writeEmptyElement(namespace, name);
        }
    }

    /**
     * Write an XML attribute with an optional namespace.
     */
    private static void writeAttribute(XMLStreamWriter writer, String name, String value,
        String namespace) throws XMLStreamException {
        if (namespace == null) {
            writer.writeAttribute(name, value);
        }
        else {
            writer.writeAttribute(namespace, name, value);
        }
    }

    // Helper formatter for BigDecimal, otherwise the digits will not be printed on integers
    private static String getBigDecimalValue(BigDecimal input) {
        NumberFormat numberFormat = NumberFormat.getNumberInstance(Locale.ENGLISH);
        numberFormat.setGroupingUsed(false);
        numberFormat.setMinimumFractionDigits(1);
        return numberFormat.format(input);
    }
}
