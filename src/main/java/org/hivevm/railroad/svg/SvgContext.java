// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.svg;

import org.hivevm.railroad.diagram.LayoutData;
import org.hivevm.railroad.diagram.Railroad;

public record SvgContext(int xOffset, int yOffset, SvgLayout layout, SvgDiagram.SvgContent content) {

    public SvgContext create(int xOffset, int yOffset) {
        return new SvgContext(xOffset, yOffset, layout, content);
    }

    public LayoutData getLayoutData(Railroad elem) {
        return layout.getLayoutData(elem);
    }
}
