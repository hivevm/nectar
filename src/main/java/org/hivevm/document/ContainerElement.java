// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;

import java.util.List;

/// Represents a document element that can contain other document elements as children. This
/// interface extends the DocumentElement interface, meaning any implementation of this interface is
/// a type of document element and adheres to the same contract.
///
/// Implementations of this interface may include elements such as blocks, lists, or tables that
/// group other document elements hierarchically.
///
/// Subclasses of this interface should ensure that modifications to the list of children adhere to
/// the expected behaviors for their specific document structure.
///
/// Responsibilities:
/// - Manage child document elements.
/// - Provide access to the list of children.
/// - Allow adding and removing of child elements.
public interface ContainerElement extends DocumentElement {

    List<DocumentElement> getChildren();

    interface Builder {

        Paragraph.Builder addParagraph();

        CodeBlock.Builder addCodeBlock(String code, String language);

        Block.Builder addBlock(String cssClass);

        ListElement.Builder addList(ListElement.ListType type);
    }
}