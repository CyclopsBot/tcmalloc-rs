"""
# **Oxlint**
Utilities for running Oxlint within bazel.
"""

load("@npm//:oxlint/package_json.bzl", oxlint_bin = "bin")

def oxlint(name, srcs, size = "small", **kwargs):
    """
    Runs Oxlint as a test bazel target.

    #### **Example**
    ```starlark
    oxlint(
        name = "lint",
        srcs = ["file1", "file2"],
    )
    ```

    Args:
        name: (String). Name of the target.
        srcs: (Label). Files to check.
        size: (String). Size of the test.
        **kwargs: Additional arguments.
    """
    oxlint_bin.oxlint_test(
        name = name,
        args = [
            "--deny=all",
            "--react-perf-plugin",
            "--nextjs-plugin",
            "--import-plugin",
            "--promise-plugin",
            "--jsx-a11y-plugin",
            "--tsconfig=./tsconfig.json",
            "--allow=no-default-export",
            "--allow=explicit-function-return-type",
            "--allow=sort-keys",
            "./",
        ],
        data = srcs,
        chdir = native.package_name(),
        size = size,
        **kwargs
    )
