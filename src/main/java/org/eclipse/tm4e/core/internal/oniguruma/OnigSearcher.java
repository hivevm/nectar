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

import java.util.List;

import org.jspecify.annotations.Nullable;

/**
 * @see <a href="https://github.com/atom/node-oniguruma/blob/master/src/onig-searcher.cc">
 *      github.com/atom/node-oniguruma/blob/master/src/onig-searcher.cc</a>
 */
final class OnigSearcher {

	private final List<OnigRegExp> regExps;

	OnigSearcher(final List<String> regExps) {
		this.regExps = regExps.stream().map(OnigRegExp::new).toList();
	}

	@Nullable
	OnigResult search(String source, int charOffset) {
		int bestLocation = 0;
		OnigResult bestResult = null;
		int index = 0;

		for (var regExp : regExps) {
			var result = regExp.search(source, charOffset);
			if (result != null && result.hasMatch()) {
				var location = result.locationAt(0);

				if (bestResult == null || location < bestLocation) {
					bestLocation = location;
					bestResult = result;
					bestResult.setIndex(index);
				}

				if (location == charOffset) {
					break;
				}
			}
			index++;
		}
		return bestResult;
	}
}
