// Copyright 2025 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.util;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStream;
import java.net.URI;
import java.net.URISyntaxException;
import java.util.Base64;
import java.util.Objects;

/**
 * The {@link DataUri} provides utility methods to convert between bytes and data URIs.
 */
public abstract class DataUri {

    private static final String EMPTY =
        "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=";

    /**
     * Avoid an instance of {@link DataUri}.
     */
    private DataUri() {
    }

    /**
     * Get the {@link InputStream}.
     */
    private static InputStream getInputStream(String uri) throws FileNotFoundException {
        return new File(uri).exists() ? new FileInputStream(uri)
            : DataUri.class.getResourceAsStream(uri);
    }

    /**
     * Converts the path to a URI.
     */
    public static InputStream toInputStream(String path) {
        return DataUri.class.getResourceAsStream("/" + path.substring(1));
    }

    /**
     * Converts the path to a URI.
     */
    public static InputStream toInputStream(String path, File workingDir) throws IOException {
        return path.startsWith(":") ? DataUri.class.getResourceAsStream("/" + path.substring(1))
            : new FileInputStream(
                path.startsWith("/") ? new File(path) : new File(workingDir, path));
    }

    /**
     * Converts the path to a URI.
     */
    public static URI toURI(String path, File workingDir) {
        if (path.startsWith(":")) {
            try {
                return Objects.requireNonNull(DataUri.class.getResource("/" + path.substring(1)))
                    .toURI();
            } catch (URISyntaxException e) {
                e.printStackTrace();
            }
        }
        return path.startsWith("/") ? new File(path).toURI() : new File(workingDir, path).toURI();
    }

    /**
     * Load an image resource from {@link ClassLoader} as DataURL.
     */
    public static String loadImage(String resource) {
        var contentType = resource.toLowerCase().endsWith(".png") ? "png" : "jpeg";
        try (var stream = DataUri.getInputStream(resource)) {
            var bytes = stream.readAllBytes();
            var base64 = Base64.getEncoder().encodeToString(bytes);
            return String.format("data:image/%s;base64,%s", contentType, base64);
        } catch (IOException e) {
        }
        return DataUri.EMPTY;
    }

    /**
     * Load an image resource from {@link ClassLoader} as DataURL.
     */
    public static String loadImage(URI uri) {
        var contentType = uri.toString().toLowerCase().endsWith(".png") ? "png" : "jpeg";
        try (var stream = uri.toURL().openStream()) {
            var bytes = stream.readAllBytes();
            var base64 = Base64.getEncoder().encodeToString(bytes);
            return String.format("data:image/%s;base64,%s", contentType, base64);
        } catch (IOException e) {
        }
        return DataUri.EMPTY;
    }
}
