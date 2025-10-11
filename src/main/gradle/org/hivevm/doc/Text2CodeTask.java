// Copyright 2024 HiveVM.ORG. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc;

import org.gradle.api.DefaultTask;
import org.gradle.api.file.DirectoryProperty;
import org.gradle.api.tasks.InputDirectory;
import org.gradle.api.tasks.Optional;
import org.gradle.api.tasks.OutputDirectory;
import org.gradle.api.tasks.TaskAction;
import org.gradle.api.tasks.options.Option;
import org.hivevm.util.text2svg.TextMateRenderer;

import javax.inject.Inject;
import java.io.*;
import java.util.Objects;

/**
 * The {@link Text2CodeTask} class.
 */
public abstract class Text2CodeTask extends DefaultTask {

    private static final String HEADER = """
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
            </head>
            <body>
            """;

    private static final String FOOTER = """
            </body>
            </html>
            """;

    @Inject
    public Text2CodeTask() {
        setGroup("HiveVM");
        setDescription("Formats Source Code from text");
    }

    @Optional
//    @Incremental
    @InputDirectory
//    @PathSensitive(PathSensitivity.RELATIVE)
    @Option(option = "input", description = "The input directory")
    public abstract DirectoryProperty getInput();

    @Optional
    @OutputDirectory
    @Option(option = "output", description = "The output directory")
    public abstract DirectoryProperty getOutput();

    @TaskAction
    public void process(/*InputChanges changes*/) {
//        System.out.println(
//                changes.isIncremental() ? "Executing incrementally" : "Executing non-incrementally");

        var currentDir = getProject().getProjectDir().getAbsoluteFile();
        var workingDir = getInput().getAsFile().getOrElse(currentDir);
        var targetDir = getOutput().getAsFile().getOrElse(workingDir);

        for (var file : Objects.requireNonNull(workingDir.listFiles(File::isFile))) {
            try {
                render(file, targetDir);
            } catch (Throwable e) {
                getLogger().warn("Error processing Source file: '{}'", file.getAbsolutePath());
                getLogger().error("Error processing Source file", e);
            }
        }
    }

    private void render(File file, File targetDir) throws IOException {
        var name = file.getName();
        var filetype = name.substring(name.lastIndexOf('.') + 1);
        var filename = name.substring(0, name.length() - filetype.length() - 1);

        var target = new File(targetDir, filename + ".html");

        getLogger().warn("Rendering '{}' to '{}'", file.getPath(), target.getPath());

        try (var response = new FileOutputStream(target);
             var request = new FileInputStream(file)) {
            var text = new String(request.readAllBytes());
            var writer = new PrintWriter(response);

            writer.print(HEADER);
            writer.println("<pre style=\"background-color:#ffffff;color:#1f2328; border: 1px solid #999; border-radius: 5px; padding: 0 15px;\">");
            writer.println("<code>");

            try {
                writer.print("<span class=\"line\">");
                var firstLine = TextMateRenderer.render(text, filetype).getFirst();
                for (var span : firstLine.span()) {
                    if (span.text() != null && !span.text().isEmpty()) {
                        writer.print(String.format("<span class=\"%s\">", span.scope()));
                        writer.print(span.text());
                        writer.print("</span>");
                    } else if (span.style() != null) {
                        writer.print(String.format("<span style=\"color: %s\">", span.style().color()));
                    } else if (span.isNewLine()) {
                        writer.println("</span>");
                        writer.print("<span class=\"line\">");
                    } else if (span.isEnd()) {
                        writer.print("</span>");
                    }
                }
                writer.println("</span>");
            } catch (Exception e) {
                getLogger().error("Error rendering file", e);
            }

            writer.println("</code>");
            writer.println("</pre>");
            writer.println("<pre style=\"background-color:#ffffff;color:#1f2328; border: 1px solid #999; border-radius: 5px; padding: 0 15px;\">");
            writer.println("<code>");

            for (var line : text.split("\n")) {
                writer.print("<span class=\"line\">");
                writer.print(line);
                writer.println("</span>");
            }

            writer.println("</code>");
            writer.println("</pre>");
            writer.print(FOOTER);
            writer.flush();
        }
    }
}
