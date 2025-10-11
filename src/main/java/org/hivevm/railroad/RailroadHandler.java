// Copyright (C) 2025 HiveVM.org
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.hivevm.railroad;

import org.hivevm.railroad.bnf.BNFParser;
import org.hivevm.railroad.bnf.BNFWriter;
import org.hivevm.railroad.diagram.Railroad;
import org.hivevm.railroad.grammar.Grammar;
import org.hivevm.railroad.grammar.Rule;
import org.hivevm.railroad.svg.SvgDiagram;
import org.hivevm.railroad.svg.SvgLayout;
import org.hivevm.util.RequestHandler;

import java.io.IOException;
import java.util.stream.Collectors;

/**
 * The {@link RailroadHandler} class.
 */
public interface RailroadHandler {

    RequestHandler<Grammar, String, Object> TO_GRAMMAR = (i, c) -> BNFParser.parse(i);
    RequestHandler<String, Grammar, Object> TO_SVG =
            (i, c) -> i.rules().stream()
                    .map(Rule::expression)
                    .map(Railroad::toRailroad)
                    .map(SvgDiagram::toDiagram)
                    .map(d -> d.toSVG(new SvgLayout())).collect(Collectors.joining("\n"));
    RequestHandler<String, Grammar, Object> TO_BNF = (i, c) -> BNFWriter.toBNF(i);

    RequestHandler<String, String, Object> BNF_TO_SVG =
            (i, c) -> TO_SVG.handleRequest(TO_GRAMMAR.handleRequest(i, c), c);
}
