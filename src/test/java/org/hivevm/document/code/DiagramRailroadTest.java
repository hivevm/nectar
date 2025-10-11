// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document.code;

import org.hivevm.util.text2svg.RailroadRenderer;

/**
 * The {@link DiagramRailroadTest} class.
 */
public class DiagramRailroadTest extends DiagramTest {

    public DiagramRailroadTest() {
        super("railroad", (i, o) -> {
            var text = new String(i.readAllBytes());
            o.write(RailroadRenderer.render(text));
        });
    }
}