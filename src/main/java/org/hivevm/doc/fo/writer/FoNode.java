// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

import java.io.Closeable;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import org.hivevm.doc.fo.writer.set.Fo;
import org.hivevm.util.xml.XmlBuilder;

/**
 * The {@link FoNode} is a builder implementation for Apache Formating Objects.
 */
public class FoNode implements Fo, Closeable, Iterable<FoNode> {

    private final FoNode     parent;
    private final XmlBuilder builder;

    private final Map<String, String> attributes = new LinkedHashMap<>();
    private final List<FoNode>        children   = new ArrayList<>();


    private boolean isClosed;

    /**
     * Constructs an instance of {@link FoNode}.
     */
    protected FoNode(String name, XmlBuilder builder) {
        this.parent = null;
        this.builder = builder;
        builder.push(name);
    }

    /**
     * Constructs an instance of {@link FoNode}.
     */
    public FoNode(String name, FoNode parent) {
        this.parent = parent;
        this.builder = parent.builder;
        this.parent.addChild(this);
        if (!(this instanceof FoText))
            builder.push(name);
    }

    private void addChild(FoNode child) {
        forEach(FoNode::close);
        this.children.add(child);
    }

    /**
     * Set an attribute.
     */
    public final FoNode set(String name, String value) {
        if (!this.children.isEmpty())
            throw new IllegalArgumentException();

        if (value != null) {
            this.attributes.put(name, value);
            this.builder.set(name, value);
        }

        return this;
    }

    /**
     * Set the ID.
     */
    public final FoNode setId(String id) {
        return set("id", id);
    }

    /**
     * Add a new child text {@link FoNode}.
     */
    public final FoNode addText(String text) {
        new FoText(text, this);
        return this;
    }

    private class FoText extends FoNode {

        private final String  text;
        private       boolean isClosed;

        public FoText(String name, FoNode parent) {
            super(name, parent);
            this.text = name;
        }

        @Override
        public void close() {
            if (isClosed)
                return;
            builder.addContentText(text);
            isClosed = true;
        }
    }

    /**
     * Returns an iterator over child {@link FoNode}.
     */
    @Override
    public final Iterator<FoNode> iterator() {
        return this.children.iterator();
    }

    /**
     * Creates a new instance of a {@link FoNode}.
     */
    public final FoNode create(String name) {
        return new FoNode(name, this);
    }

    /**
     * Build the String.
     */
    @Override
    public void close() {
        if (isClosed)
            return;
        forEach(FoNode::close);
        builder.build();
        isClosed = true;
    }
}
