// Copyright 2024 HiveVM.ORG. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc;

import org.gradle.api.DefaultTask;
import org.gradle.api.file.DirectoryProperty;
import org.gradle.api.provider.Property;
import org.gradle.api.tasks.Input;
import org.gradle.api.tasks.InputDirectory;
import org.gradle.api.tasks.Optional;
import org.gradle.api.tasks.TaskAction;
import org.gradle.api.tasks.options.Option;
import org.hivevm.doc.fo.pdf.PdfRenderer;
import org.hivevm.doc.template.Template;
import org.hivevm.util.ReplacerRequestHandler;
import org.hivevm.util.lambda.RequestStreamBuilder;
import org.hivevm.util.lambda.RequestStreamHandler;

import javax.inject.Inject;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.util.HashMap;
import java.util.Map;
import java.util.regex.Pattern;

/**
 * The {@link PdfTask} class.
 */
public abstract class PdfTask extends DefaultTask {

    @Inject
    public PdfTask() {
        setGroup("HiveVM");
        setDescription("Creates PDF's from Markdown");
    }

    @InputDirectory
    @Optional
    @Option(option = "source", description = "The input folder containing markdown files.")
    public abstract DirectoryProperty getSource();

    @Input
    @Optional
    @Option(option = "template", description = "The template.")
    public abstract Property<String> getTemplate();

    @TaskAction
    public void process() {
        var workingDir = getProject().getProjectDir().getAbsoluteFile();
        var config = getProject().getExtensions().findByType(GradleConfig.class);
        var source = getFile((config == null) ? null : config.source, getProject().getProjectDir());

        String templatePath = getTemplate().getOrElse(
                (config == null || config.template == null) ? ":default.ui.xml" : config.template);
        if (templatePath.startsWith(":") && templatePath.endsWith(":")) {
            templatePath = String.format("%s.ui.xml",
                    templatePath.substring(0, templatePath.length() - 1).toLowerCase());
        } else if (!templatePath.isEmpty() && !templatePath.startsWith(":")
                && !templatePath.startsWith("/")) {
            var file = new File(workingDir, templatePath);
            workingDir = file.getParentFile();
            templatePath = file.getName();
        }

        Map<String, String> props = new HashMap<>();
        System.getProperties().entrySet().stream().filter(e -> e.getValue() != null)
                .forEach(e -> props.put((String) e.getKey(), (String) e.getValue()));

        try {
            Template template = Template.parse(templatePath, workingDir);

            RequestStreamBuilder builder = new RequestStreamBuilder();
            builder.append(new ReplacerRequestHandler(props));
            builder.append(new PdfRenderer(template));
            RequestStreamHandler mdHandler = builder.build();

            builder = new RequestStreamBuilder();
            builder.append(new ReplacerRequestHandler(props));
            builder.append(new PdfRenderer(template));
            RequestStreamHandler asciiHandler = builder.build();

            File folder = source.isDirectory() ? source : source.getParentFile();
            String file = source.isDirectory() ? "*.{md,adoc}" : source.getName();
            String text = file.replace(".", "\\.").replace("{", "(").replace("}", ")")
                    .replace(",", "|")
                    .replace("*", ".+");
            Pattern pattern = Pattern.compile(text);

            var targetDir = getProject().getBuildDir();
            if (!targetDir.exists())
                targetDir.mkdirs();

            for (File input : folder.listFiles(f -> pattern.matcher(f.getName()).find())) {
                var handler = input.getName().endsWith(".md") ? mdHandler : asciiHandler;
                var output = new File(targetDir, input.getName() + ".pdf");
                try (FileOutputStream ostream = new FileOutputStream(output);
                     InputStream istream = new FileInputStream(input)) {
                    handler.handleRequest(istream, ostream, input.getParentFile());
                }
            }
        } catch (Exception e) {
            getLogger().error("Failed to generate a PDF", e);
        }
    }

    protected File getFile(String pathname, File folder) {
        if (pathname == null) {
            return folder;
        }

        File file = new File(pathname);
        if (file.isAbsolute()) {
            return file;
        }

        File projectDir = getProject().getProjectDir();
        return new File(projectDir, pathname);
    }
}
