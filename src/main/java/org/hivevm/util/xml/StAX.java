// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause
package org.hivevm.util.xml;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.Writer;
import java.lang.reflect.Proxy;
import java.nio.charset.StandardCharsets;
import java.util.HashMap;
import java.util.Iterator;
import java.util.Map;
import java.util.function.Consumer;
import javax.xml.namespace.QName;
import javax.xml.stream.XMLEventReader;
import javax.xml.stream.XMLInputFactory;
import javax.xml.stream.XMLOutputFactory;
import javax.xml.stream.XMLStreamConstants;
import javax.xml.stream.XMLStreamException;
import javax.xml.stream.XMLStreamReader;
import javax.xml.stream.XMLStreamWriter;
import javax.xml.stream.events.Attribute;
import javax.xml.stream.events.EndElement;
import javax.xml.stream.events.StartElement;
import javax.xml.stream.events.XMLEvent;
import javax.xml.transform.TransformerException;
import javax.xml.transform.TransformerFactory;
import javax.xml.transform.dom.DOMSource;
import javax.xml.transform.sax.SAXResult;
import javax.xml.transform.stream.StreamResult;
import javax.xml.transform.stream.StreamSource;
import org.w3c.dom.Document;
import org.xml.sax.ContentHandler;


/**
 * The {@link StAX} utility provides an XML parser.
 */
public abstract class StAX {

    private static final XMLInputFactory    READER_FACTORY = XMLInputFactory.newInstance();
    private static final XMLOutputFactory   WRITER_FACTORY = XMLOutputFactory.newInstance();
    private static final TransformerFactory TRANSFORMER    = TransformerFactory.newInstance();

    static {
        StAX.READER_FACTORY.setProperty(XMLInputFactory.IS_NAMESPACE_AWARE, Boolean.TRUE);
        StAX.WRITER_FACTORY.setProperty(XMLOutputFactory.IS_REPAIRING_NAMESPACES, Boolean.TRUE);
        if (StAX.WRITER_FACTORY.isPropertySupported("com.ctc.wstx.useDoubleQuotesInXmlDecl")) {
            StAX.WRITER_FACTORY.setProperty("com.ctc.wstx.useDoubleQuotesInXmlDecl", Boolean.TRUE);
        }
        StAX.TRANSFORMER.setAttribute("indent-number", 2);
    }

    /**
     * Constructs a(n) {@link StAX} object.
     */
    private StAX() {
    }

    /**
     * Creates an instance of {@link XMLStreamReader}.
     */
    public static XMLStreamReader createReader(InputStream stream) throws XMLStreamException {
        return StAX.READER_FACTORY.createXMLStreamReader(stream, StandardCharsets.UTF_8.name());
    }

    /**
     * Creates an instance of {@link XMLStreamWriter}.
     */
    public static XMLStreamWriter createWriter(OutputStream stream) throws XMLStreamException {
        return StAX.WRITER_FACTORY.createXMLStreamWriter(stream, StandardCharsets.UTF_8.name());
    }

    /**
     * Creates an instance of {@link XMLStreamWriter}.
     */
    public static XMLStreamWriter createWriter(Writer writer) throws XMLStreamException {
        return StAX.WRITER_FACTORY.createXMLStreamWriter(writer);
    }

    /**
     * Creates an instance of {@link XMLStreamWriter}.
     */
    public static XMLStreamWriter formatted(XMLStreamWriter writer, int indent) {
        return (indent == 0) ? writer
            : (XMLStreamWriter) Proxy.newProxyInstance(XMLStreamWriter.class.getClassLoader(),
                new Class[]{XMLStreamWriter.class}, new XmlPrettyWriter(writer, indent));
    }

    /**
     * Create a {@link StAX} from a resource.
     */
    public static void parse(InputStream stream, Handler handler) throws IOException {
        try {
            XMLEventReader reader = StAX.READER_FACTORY.createXMLEventReader(stream);
            StringBuilder textBuffer = null;
            while (reader.hasNext()) {
                XMLEvent event = reader.nextEvent();
                switch (event.getEventType()) {
                    case XMLStreamConstants.START_ELEMENT:
                        StartElement start = event.asStartElement();
                        handler.handleEvent(start.getName().getLocalPart(),
                            StAX.parseAttributes(start));
                        textBuffer = new StringBuilder();
                        break;
                    case XMLStreamConstants.END_ELEMENT:
                        EndElement end = event.asEndElement();
                        handler.handleEvent(end.getName().getLocalPart(),
                            textBuffer == null ? null : textBuffer.toString());
                        textBuffer = null;
                        break;
                    case XMLStreamConstants.CDATA:
                    case XMLStreamConstants.CHARACTERS:
                        if (textBuffer != null) {
                            textBuffer.append(event.asCharacters().getData());
                        }
                        break;
                    default:
                        break;
                }
            }
        } catch (XMLStreamException e) {
            throw new IOException(e);
        }
    }

    public static void transform(InputStream stream, ContentHandler handler) throws IOException {
        try {
            // Setup JAXP using identity transformer
            var transformer = TransformerFactory.newInstance().newTransformer();
            // Setup input and output for XSLT transformation
            var result = new SAXResult(handler);
            // Start XSLT transformation and FOP processing
            transformer.transform(new StreamSource(stream), result);
        } catch (TransformerException e) {
            throw new IOException(e);
        }
    }

    public static void transform(OutputStream stream, Document document) throws IOException {
        try {
            // Setup JAXP using identity transformer
            var transformer = TransformerFactory.newInstance().newTransformer();
            transformer.transform(new DOMSource(document), new StreamResult(stream));
        } catch (TransformerException e) {
            throw new IOException(e);
        }
    }

    /**
     * Parses the attributes from {@link StartElement}.
     */
    private static Attributes parseAttributes(StartElement elem) {
        Map<String, String> attributes = new HashMap<>();
        Iterator<Attribute> iterator = elem.getAttributes();
        while (iterator.hasNext()) {
            Attribute attr = iterator.next();
            QName name = attr.getName();
            String value = attr.getValue();
            attributes.put(name.getLocalPart(), value);
        }
        return new Attributes(attributes);
    }

    /**
     * The {@link Handler} class.
     */
    public interface Handler {

        void handleEvent(String name, Attributes attributes);

        void handleEvent(String name, String content);
    }

    public static class Attributes {

        private final Map<String, String> attributes;

        private Attributes(Map<String, String> attributes) {
            this.attributes = attributes;
        }

        public final boolean isSet(String name) {
            return this.attributes.containsKey(name);
        }

        public final String get(String name) {
            return this.attributes.get(name);
        }

        public final String get(String name, String value) {
            return isSet(name) ? this.attributes.get(name) : value;
        }

        public final boolean getBool(String name) {
            return Boolean.parseBoolean(get(name, "false"));
        }

        public final void onAttribute(String name, Consumer<String> consumer) {
            if (this.attributes.containsKey(name)) {
                consumer.accept(this.attributes.get(name));
            }
        }
    }
}
