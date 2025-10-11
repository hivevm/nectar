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

import org.joni.Region;

import java.util.regex.Matcher;

/**
 * @see <a href="https://github.com/atom/node-oniguruma/blob/master/src/onig-result.cc">
 *      github.com/atom/node-oniguruma/blob/master/src/onig-result.cc</a>
 */
public final class OnigResult {

	private int indexInScanner;
	private final Region region;
    private final Matcher matcher;

	OnigResult(Matcher matcher, Region region) {
		this.region = region;
        this.matcher = matcher;
		this.indexInScanner = -1;
	}

	public int getIndex() {
		return indexInScanner;
	}

	void setIndex(final int index) {
		indexInScanner = index;
	}

    public boolean hasMatch() {
        return count() > 0;
    }

	public int count() {
		return matcher.groupCount() + 1;
	}

    public int locationAt(int index) {
        var bytes = Math.max(region.getBeg(index), 0);
        var chars = Math.max(matcher.start(index), 0);
        var count = matcher.groupCount();
        var text = matcher.group(index);
        return bytes;
    }

	public int lengthAt(int index) {
        var bytes = Math.max(region.getEnd(index) - region.getBeg(index), 0);
        var chars = Math.max(matcher.end(index) - matcher.start(index), 0);
		return bytes;
	}

	@Override
	public String toString() {
		return "OnigResult [indexInScanner=" + indexInScanner + ", region=" + region + "]";
	}
}
