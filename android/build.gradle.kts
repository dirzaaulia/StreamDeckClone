plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.compose.compiler) apply false
    alias(libs.plugins.kotlin.serialization) apply false
}

val projectDir = layout.projectDirectory

tasks.register("checkLineBudget") {
    group = "verification"
    description = "Enforces clean architecture line limits (max 250 lines per Kotlin file)."
    val rootDirFile = projectDir.asFile
    doLast {
        val maxLines = 250
        val files = rootDirFile.walkTopDown()
            .filter { it.extension == "kt" }
            .filterNot { it.path.contains("build") || it.path.contains(".gradle") }
            .filter { it.readLines().size > maxLines }
            .toList()

        if (files.isNotEmpty()) {
            val message = buildString {
                appendLine("\n" + "=".repeat(75))
                appendLine("❌ BUILD BLOCKED: LINE BUDGET VIOLATION (Max allowed: $maxLines lines)")
                appendLine("=".repeat(75))
                files.forEach { file ->
                    val lineCount = file.readLines().size
                    appendLine("  [FAIL] ${file.relativeTo(rootDirFile).path} -> $lineCount lines")
                }
                appendLine("=".repeat(75))
            }
            throw GradleException(message)
        }
    }
}