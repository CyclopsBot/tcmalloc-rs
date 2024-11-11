"""
# **Svelte**
Rules for interacting with svelte through bazel.
"""

load("@aspect_rules_js//js:defs.bzl", "js_run_binary", "js_run_devserver")
load("//libs/bazel/rules/private:vite.bzl", "vite")

def _impl(name, srcs, deps, package_json, tsconfig, config, visibility):
    vite(
        name = name,
        srcs = srcs,
        out = [".svelte-kit"],
        config = config,
        package_json = package_json,
        deps = deps,
        tsconfig = tsconfig,
        visibility = visibility,
    )
    js_run_binary(
        name = name + ".storybook",
        srcs = srcs + deps + package_json + config + tsconfig,
        args = ["build"],
        chdir = native.package_name(),
        tool = "//tools/javascript:storybook_binary",
        out_dirs = ["storybook-static"],
        mnemonic = "StorybookBuild",
        progress_message = "Building %{input}",
        visibility = visibility,
    )
    js_run_devserver(
        name = name + ".storybook.dev",
        data = srcs + deps + package_json + config,
        chdir = native.package_name(),
        args = ["dev"],
        tool = "//tools/javascript:storybook_binary",
        visibility = visibility,
    )
    js_run_binary(
        name = name + ".check",
        srcs = srcs + deps + package_json + config + tsconfig + [":{}.build".format(name)],
        args = ["--tsconfig", "./tsconfig.json"],
        chdir = native.package_name(),
        tool = "//tools/javascript:svelte_check_binary",
        out_dirs = ["dist"],
        mnemonic = "SvelteCheck",
        progress_message = "Checking %{input}",
        visibility = visibility,
        testonly = True,
    )
    native.alias(
        name = name + ".build",
        actual = name + "_build",
        visibility = visibility,
    )
    native.alias(
        name = name + ".dev",
        actual = name + "_dev",
        visibility = visibility,
    )

svelte = macro(
    implementation = _impl,
    doc = """
    Runs Svelte.

    #### **Example**
    ```starlark
    svelte(
        name = "example",
        srcs = ["test.txt", "test.js"],
        deps = ["//:node_modules"],
        config = [":svelte.config.js", "vite.config.ts"],
        package_json = [":package_json"],
        tsconfig = [":tsconfig"],
    )
    ```
    """,
    attrs = {
        "config": attr.label_list(
            mandatory = True,
            configurable = False,
            doc = "Vite and Svelte config to passthrough.",
        ),
        "deps": attr.label_list(
            mandatory = True,
            configurable = False,
            doc = "Dependencies you wish for vite to use.",
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
