"""
# **Vite**
Rules for interacting with vite through bazel.
"""

load("@aspect_rules_js//js:defs.bzl", "js_run_binary", "js_run_devserver")

def _impl(name, srcs, deps, package_json, tsconfig, config, visibility, out):
    js_run_binary(
        name = name + "_build",
        srcs = srcs + deps + package_json + config + tsconfig,
        args = ["build"],
        chdir = native.package_name(),
        tool = "//tools/javascript:vite_binary",
        out_dirs = out,
        mnemonic = "ViteBuild",
        progress_message = "Compiling %{input}",
        visibility = visibility,
    )
    js_run_devserver(
        name = name + "_dev",
        data = srcs + deps + package_json + config + tsconfig,
        chdir = native.package_name(),
        tool = "//tools/javascript:vite_binary",
        visibility = visibility,
    )

vite = macro(
    implementation = _impl,
    doc = """
    Runs vite.

    #### **Example**
    ```starlark
    vite(
        name = "example",
        srcs = ["test.txt", "test.js"],
        deps = ["//:node_modules"],
        config = [""],
        package_json = [""],
        tsconfig = [":tsconfig"],
        out = [".sveltekit"],
    )
    ```
    """,
    attrs = {
        "config": attr.label_list(
            mandatory = True,
            configurable = False,
            doc = "Vite config to passthrough.",
        ),
        "deps": attr.label_list(
            mandatory = True,
            configurable = False,
            doc = "Dependencies you wish for vite to use.",
        ),
        "out": attr.string_list(
            configurable = False,
            doc = "Resulting directory that vite produces.",
        ),
        "package_json": attr.label_list(
            configurable = False,
            mandatory = True,
            doc = "The package.json(s) you wish to expose to vite",
        ),
        "srcs": attr.label_list(
            mandatory = True,
            configurable = False,
            doc = "Files you wish for vite to see.",
        ),
        "tsconfig": attr.label_list(
            configurable = False,
            mandatory = False,
            doc = "placeholder text",
        ),
    },
)
