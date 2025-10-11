// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc.template;

import java.io.File;
import java.net.URI;
import org.hivevm.util.DataUri;

/// The background defines a background color or an image.
public sealed interface Background permits Background.Color, Background.Image {

  String get();

  boolean isImage();

  static Background of(String value, File workingDir) {
    return value.startsWith("#") ? new Color(value) : new Image(DataUri.toURI(value, workingDir));
  }

  record Color(String color) implements Background {

    public boolean isImage() {
      return false;
    }

    public String get() {
      return color;
    }
  }

  record Image(URI uri) implements Background {

    public boolean isImage() {
      return true;
    }

    public String get() {
      return DataUri.loadImage(uri);
    }
  }
}
