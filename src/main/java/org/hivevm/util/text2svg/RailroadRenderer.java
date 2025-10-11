// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util.text2svg;

import org.hivevm.railroad.RailroadHandler;

import java.io.IOException;

public class RailroadRenderer {

    public static byte[] render(String text) throws IOException {
        return RailroadHandler.BNF_TO_SVG.handleRequest(text, null).getBytes();
    }
}
