// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

///  Defines an interface for a document visitor in the Visitor design pattern. The purpose of the
/// visitor is to perform operations on different types of document elements without modifying their
/// classes.
///
///  New types of document elements can be added easily by introducing a new visit method, adhering
/// to the Open/Closed principle.
public interface DocumentVisitor<C> {

    void visit(Heading heading, C context);

    void visit(Paragraph paragraph, C context);

    void visit(PageBreak pageBreak, C context);

    void visit(Image image, C context);

    void visit(CodeBlock codeBlock, C context);

    void visit(Table table, C context);

    void visit(ListElement list, C context);

    void visit(Block block, C context);
}