// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.diagram;

/**
 * The {@link LayoutData} provides rendering information for the RailRoad diagram.
 */
public record LayoutData(int width, int height, int connectorOffset, int fontYOffset,
                         String cardinalitiesText, int cardinalitiesWidth) {

    LayoutData(int width, int height, int connectorOffset, int fontYOffset) {
        this(width, height, connectorOffset, fontYOffset, null, 0);
    }

    LayoutData(int width, int height, int connectorOffset) {
        this(width, height, connectorOffset, 0);
    }

    public static LayoutData create(Railroad railroad, LayoutContext context) {
        var data = railroad.accept(new LayoutDataBuilder(), context);
        context.setLayoutData(railroad, data);
        return data;
    }
}
