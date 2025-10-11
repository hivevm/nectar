// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Defines a Unit implementation, which is basically a numeric value and the definition of an unit.
 *
 * @param value
 * @param unit
 */
public record Unit(double value, String unit) {

    private static final Pattern UNIT = Pattern.compile("([\\d\\-\\.]+)([a-zA-Z]*)", Pattern.CASE_INSENSITIVE);

    public static final Unit NIL = new Unit(0, null);

    public String asString() {
        if (this == NIL)
            return null;
        if (unit == null)
            return Double.toString(value);
        return value + unit;
    }

    public static Unit parse(String value) {
        Matcher matcher = UNIT.matcher(value);
        return matcher.find() ? new Unit(Double.valueOf(matcher.group(1)), matcher.group(2)) : Unit.NIL;
    }
}
