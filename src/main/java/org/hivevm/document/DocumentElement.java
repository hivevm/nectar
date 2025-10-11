// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.document;


/// Represents the base interface for all document elements. A document element is a component that
/// can participate in a document structure. Each implementation of this interface provides specific
/// behaviors and properties related to its type. Examples of document elements are paragraphs,
/// headings, tables, and more.
///
///  Responsibilities:
///  - Provide a unique identifier for the document element.
///  - Support the Visitor pattern for traversing and processing document elements.
///
///  Methods:
///  1. getId(): Returns the unique identifier of the document element.
///  2. accept(DocumentVisitor): Accepts a visitor to perform operations on the document element.
///
///  This interface forms the foundation for different types of document elements, enabling a
/// unified approach for managing and processing diverse components within a document structure.
public interface DocumentElement {

    String id();

    <C> void accept(DocumentVisitor visitor, C context);
}