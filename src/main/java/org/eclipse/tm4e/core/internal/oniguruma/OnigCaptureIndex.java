/**
 * Copyright (c) 2015-2017 Angelo ZERR.
 * This program and the accompanying materials are made
 * available under the terms of the Eclipse Public License 2.0
 * which is available at https://www.eclipse.org/legal/epl-2.0/
 *
 * SPDX-License-Identifier: EPL-2.0
 *
 * Initial code from https://github.com/atom/node-oniguruma
 * Initial copyright Copyright (c) 2013 GitHub Inc.
 * Initial license: MIT
 *
 * Contributors:
 * - GitHub Inc.: Initial code, written in JavaScript, licensed under MIT license
 * - Angelo Zerr <angelo.zerr@gmail.com> - translation and adaptation to Java
 */
package org.eclipse.tm4e.core.internal.oniguruma;

import org.jspecify.annotations.Nullable;

/**
 * @see <a href="https://github.com/atom/node-oniguruma/blob/0c6b95fc7d79ab7e60a7ed63df6d05677ace2642/src/onig-scanner.cc#L110">
 *      github.com/atom/node-oniguruma/blob/master/src/onig-scanner.cc#L110</a>
 */
public record OnigCaptureIndex(int start, int end) {

	private static final OnigCaptureIndex EMPTY = new OnigCaptureIndex(0, 0);

    public CharSequence subSequence(CharSequence sequence) {
        return sequence.subSequence(start, end);
    }

    public boolean isEmpty() {
        return end - start == 0;
    }

	@Override
	public int hashCode() {
		return 31 * (31 + end) + start;
	}

	@Override
	public String toString() {
		return "{"
				+ ", \"start\": " + start
				+ ", \"end\": " + end
				+ ", \"length\": " + (end - start)
				+ "}";
	}


    public static OnigCaptureIndex of(int start, int end) {
        var _start = Math.max(start, 0);
        var _end = Math.max(end, 0);
        return _start == 0 && _end == 0 ? EMPTY : new OnigCaptureIndex(_start, _end);
    }
}
