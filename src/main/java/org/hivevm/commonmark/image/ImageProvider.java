// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.commonmark.image;

import java.util.Map;
import org.commonmark.node.AbstractVisitor;
import org.commonmark.node.CustomNode;
import org.commonmark.node.Image;
import org.commonmark.node.Node;
import org.commonmark.renderer.html.AttributeProvider;

/**
 * The ImageProvider class is an implementation of the AttributeProvider interface for processing
 * and adding custom attributes to image nodes in a CommonMark document.
 * <p>
 * This class works in conjunction with other components in the API to provide extended support for
 * custom attributes associated with images. It specifically handles the attributes stored in
 * ImageAttributes nodes and attaches them to the image node before rendering.
 * <p>
 * Key roles of this class include:
 * <p>
 * - Providing a factory method `create` to instantiate instances of the ImageProvider. - Overriding
 * the `setAttributes` method to process and set attributes to an image node. - Visiting and
 * removing ImageAttributes nodes after their data has been applied.
 * <p>
 * This class is commonly used with the HtmlRenderer to control how custom image attributes are
 * rendered as part of the CommonMark document output.
 */
class ImageProvider implements AttributeProvider {

    private ImageProvider() {
    }

    public static ImageProvider create() {
        return new ImageProvider();
    }

    @Override
    public void setAttributes(Node node, String tagName, Map<String, String> attributes) {
        if (node instanceof Image) {
            node.accept(new AbstractVisitor() {

                @Override
                public void visit(CustomNode node) {
                    if (node instanceof ImageAttributes imageAttributes) {
                        attributes.putAll(imageAttributes.getAttributes());
                        // Now that we have used the image attributes we remove the node.
                        imageAttributes.unlink();
                    }
                }
            });
        }
    }
}
