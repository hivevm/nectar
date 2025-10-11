// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad.diagram;

import java.util.Arrays;
import java.util.Stack;

class RailroadBuilder {

    private final Stack<Railroad> stack = new Stack<>();

    public final RailroadBuilder add(Railroad elem) {
        stack.push(elem);
        return this;
    }

    public final RailroadBuilder addLine() {
        return add(new Railroad.Line());
    }

    public final RailroadBuilder addBreak() {
        return add(new Railroad.Break());
    }

    public final RailroadBuilder addText(Railroad.Type type, String text, String link) {
        return add(new Railroad.Text(type, text, link));
    }

    public final RailroadBuilder addLoop(int minRepetitionCount, Integer maxRepetitionCount) {
        var elem = stack.pop();
        return add(Railroad.Loop.create(elem, null, minRepetitionCount, maxRepetitionCount));
    }

    public final RailroadBuilder addLoop2(int minRepetitionCount, Integer maxRepetitionCount) {
        var loop = stack.pop();
        var elem = stack.pop();
        return add(Railroad.Loop.create(elem, loop, minRepetitionCount, maxRepetitionCount));
    }

    public final RailroadBuilder addChoice(int count) {
        var array = new Railroad[count];
        while (count > 0)
            array[--count] = stack.pop();
        return add(new Railroad.Choice(Arrays.asList(array)));
    }

    public final RailroadBuilder addSequence(int count) {
        var array = new Railroad[count];
        while (count > 0)
            array[--count] = stack.pop();
        return add(new Railroad.Sequence(Arrays.asList(array)));
    }

    public final Railroad build() {
        return stack.pop();
    }
}
