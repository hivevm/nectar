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
import org.hivevm.util.text2svg.GraphvizRenderer;
import org.hivevm.util.text2svg.RailroadRenderer;

import javax.inject.Inject;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.util.Objects;

/**
 * The {@link Text2SvgTask} class.
 */
public abstract class Text2SvgTask extends DefaultTask {

    @Inject
    public Text2SvgTask() {
        setGroup("HiveVM");
        setDescription("Creates SVG Diagrams from text");
    }

    /**
     * Checks whether the provided file is supported based on its file extension.
     * Supported file extensions are ".dot", ".plantuml", and ".railroad".
     */
    protected boolean isSupported(File file) {
        var filename = file.getName();
        return filename.endsWith(".dot") || filename.endsWith(".plantuml") || filename.endsWith(".railroad");
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

        for (var file : Objects.requireNonNull(workingDir.listFiles(this::isSupported))) {
            try {
                render(file, targetDir);
            } catch (Throwable e) {
                getLogger().warn("Error processing SVG file: '{}'", file.getAbsolutePath());
                e.printStackTrace();
            }
        }

        /**
         changes.getFileChanges(workingDir).forEach(c -> {
         if (c.getChangeType() == ChangeType.REMOVED || !c.getFile().getName().endsWith(".md"))
         return;

         File source = c.getFile();
         Resolver resolver = new Resolver.PathResolver(source.getParentFile());
         File target = new File(getOutput().get().getAsFile(), source.getName());

         getLogger().warn("Merge '{}' into '{}'", source.getAbsolutePath(),
         target.getAbsolutePath());

         try (Reader reader = new FileReader(source);
         Writer writer = new FileWriter(target)) {
         MarkdownReader md = new MarkdownReader(reader, resolver);
         writer.write(md.readAll());
         } catch (IOException e) {
         getLogger().error("Failed to merge the file", e);
         }
         });
         */
    }

    private void render(File file, File targetDir) throws IOException{
        var name = file.getName();
        var filetype = name.substring(name.lastIndexOf('.') + 1);
        var filename = name.substring(0, name.length() - filetype.length() - 1);

        var target = new File(targetDir, filename + ".svg");
        try (var request = new FileInputStream(file);
             var response = new FileOutputStream(target))
        {
            var text = new String(request.readAllBytes());
            var bytes = filetype.equals("railroad") ? RailroadRenderer.render(text) : GraphvizRenderer.render(text);
            response.write(bytes);
        }
    }
}
