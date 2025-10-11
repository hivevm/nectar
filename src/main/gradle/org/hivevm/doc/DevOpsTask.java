// Copyright 2024 HiveVM.ORG. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

package org.hivevm.doc;

import java.io.File;
import javax.inject.Inject;
import org.gradle.api.DefaultTask;
import org.gradle.api.file.DirectoryProperty;
import org.gradle.api.file.FileType;
import org.gradle.api.tasks.InputDirectory;
import org.gradle.api.tasks.OutputDirectory;
import org.gradle.api.tasks.PathSensitive;
import org.gradle.api.tasks.PathSensitivity;
import org.gradle.api.tasks.TaskAction;
import org.gradle.api.tasks.options.Option;
import org.gradle.work.ChangeType;
import org.gradle.work.Incremental;
import org.gradle.work.InputChanges;

public abstract class DevOpsTask extends DefaultTask {

    @Inject
    public DevOpsTask() {
        setGroup("HiveVM");
        setDescription("Triggers an automatic Pipeline");
    }

    @Incremental
    @InputDirectory
    @PathSensitive(PathSensitivity.NAME_ONLY)
    @Option(option = "inputDir", description = "Gets the source path")
    abstract DirectoryProperty getInputDir();

    @OutputDirectory
    @Option(option = "outputDir", description = "Gets the output path")
    abstract DirectoryProperty getOutputDir();

    @TaskAction
    void execute(InputChanges inputChanges) {
        System.out.println(inputChanges.isIncremental()
            ? "Executing incrementally"
            : "Executing non-incrementally"
        );

        inputChanges.getFileChanges(getInputDir()).forEach(change -> {
            if (change.getFileType() == FileType.DIRECTORY)
                return;

            System.out.printf("%s: %s\n", change.getChangeType(), change.getNormalizedPath());
            File targetFile = getOutputDir().file(change.getNormalizedPath()).get().getAsFile();
            if (change.getChangeType() == ChangeType.REMOVED) {
                targetFile.delete();
            }
            else {
                // targetFile.text = change.file.text.reverse()
            }
        });
    }
}