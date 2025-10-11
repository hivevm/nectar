// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

import org.hivevm.util.Unit;

/**
 * The {@link PageRenderer} class.
 */
public interface PageRenderer<B> {

    void visit(PageColumn item, B block);

    void visit(PageImage item, B block);

    void visit(PageRow item, B block);

    void visit(PageBreak item, B block);

    abstract class PageRenderable {

        private Unit top    = Unit.NIL;
        private Unit left   = Unit.NIL;
        private Unit right  = Unit.NIL;
        private Unit bottom = Unit.NIL;

        private Unit width  = Unit.NIL;
        private Unit height = Unit.NIL;


        public final String getTop() {
            return this.top.asString();
        }

        public final String getLeft() {
            return this.left.asString();
        }

        public final String getRight() {
            return this.right.asString();
        }

        public final String getBottom() {
            return this.bottom.asString();
        }

        public final String getWidth() {
            return this.width.asString();
        }

        public final String getHeight() {
            return this.height.asString();
        }

        public final void setTop(String value) {
            this.top = Unit.parse(value);
        }

        public final void setLeft(String value) {
            this.left = Unit.parse(value);
        }

        public final void setRight(String value) {
            this.right = Unit.parse(value);
        }

        public final void setBottom(String value) {
            this.bottom = Unit.parse(value);
        }

        public final void setWidth(String value) {
            this.width = Unit.parse(value);
        }

        public final void setHeight(String value) {
            this.height = Unit.parse(value);
        }

        public abstract <B> void accept(PageRenderer<B> visitor, B block);
    }
}