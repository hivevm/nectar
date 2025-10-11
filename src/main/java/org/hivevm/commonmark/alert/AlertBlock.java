// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.commonmark.alert;

import org.commonmark.node.CustomBlock;

/**
 * Represents a custom alert block within a document. The AlertBlock class extends the CustomBlock
 * class to encapsulate specific types of alert content, such as informational notes, success
 * messages, warnings, or errors.
 * <p>
 * The type of the alert is specified during instantiation and is stored as an {@link Alert}. This
 * class is used as part of the custom CommonMark parser extensions for handling alert blocks in
 * markdown content.
 */
public class AlertBlock extends CustomBlock {

    private final Alert type;

    public AlertBlock(Alert type) {
        this.type = type;
    }

    public Alert getType() {
        return this.type;
    }
}