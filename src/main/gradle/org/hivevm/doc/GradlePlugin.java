// Copyright 2024 HiveVM.org. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc;

import org.gradle.api.Plugin;
import org.gradle.api.Project;
import org.gradle.api.plugins.ExtensionContainer;

public class GradlePlugin implements Plugin<Project> {

    private static final String CONFIG = "markdown";

    @Override
    public void apply(Project project) {
        ExtensionContainer extension = project.getExtensions();
        extension.create(GradlePlugin.CONFIG, GradleConfig.class);

        project.getTasks().register("generatePdf", PdfTask.class);
        project.getTasks().register("mergeMarkdown", MergeTask.class);

        project.getTasks().register("text2svg", Text2SvgTask.class);
        project.getTasks().register("text2code", Text2CodeTask.class);
    }
}
