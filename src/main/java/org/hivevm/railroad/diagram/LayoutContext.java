// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.diagram;


import org.hivevm.railroad.Font;
import org.hivevm.railroad.Insets;

/**
 * The {@link LayoutContext} provides rendering information for the RailRoad diagram.
 */
public interface LayoutContext {

    Font getLoopFont();

    Insets getRuleInsets();

    Font getRuleFont();

    Insets getLiteralInsets();

    Font getLiteralFont();

    Insets getSpecialSequenceInsets();

    Font getSpecialSequenceFont();

    boolean isRuleTextAlignedOnBaseLine();

    void setLayoutData(Railroad elem, LayoutData data);
}
