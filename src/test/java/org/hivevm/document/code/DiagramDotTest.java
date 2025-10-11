// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document.code;

import org.hivevm.util.text2svg.GraphvizRenderer;

/**
 * The {@link DiagramDotTest} class.
 */
public class DiagramDotTest extends DiagramTest {

    public DiagramDotTest() {
        super("dot", (i, o) -> {
            var text = new String(i.readAllBytes());
            o.write(GraphvizRenderer.render(text));
        });
    }
}