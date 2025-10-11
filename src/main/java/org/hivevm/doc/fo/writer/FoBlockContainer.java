// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.fo.writer;

import org.hivevm.doc.fo.writer.set.FoBackground;
import org.hivevm.doc.fo.writer.set.FoBorder;
import org.hivevm.doc.fo.writer.set.FoFont;
import org.hivevm.doc.fo.writer.set.FoSpace;

/**
 * The {@link FoBlockContainer} class.
 */
public class FoBlockContainer extends FoNode implements FoSpace<FoBlockContainer>,
        FoBorder<FoBlockContainer>,
        FoFont<FoBlockContainer>, FoBackground<FoBlockContainer> {

    public enum Position {

        Absolute("absolute"),
        Auto("auto"),
        Fixed("fixed");

        public final String value;

        Position(String value) {
            this.value = value;
        }
    }

    /**
     * Constructs an instance of {@link FoBlockContainer}.
     */
    public FoBlockContainer(FoStaticContent content) {
        super("block-container", content);
    }

    public FoBlockContainer setAbsolute(Position position) {
        set("absolute-position", position.value);
        return this;
    }

    public FoBlockContainer setPosition(String left, String right, String top, String bottom) {
        setPositionTop(top);
        setPositionLeft(left);
        setPositionRight(right);
        setPositionBottom(bottom);
        return this;
    }

    public FoBlockContainer setPositionTop(String value) {
        set("top", value);
        return this;
    }

    public FoBlockContainer setPositionLeft(String value) {
        set("left", value);
        return this;
    }

    public FoBlockContainer setPositionRight(String value) {
        set("right", value);
        return this;
    }

    public FoBlockContainer setPositionBottom(String value) {
        set("bottom", value);
        return this;
    }

    public FoBlockContainer setWidth(String value) {
        set("width", value);
        return this;
    }

    public FoBlockContainer setHeight(String value) {
        set("height", value);
        return this;
    }

    public FoBlock addBlock() {
        return FoBlock.block(this);
    }
}
