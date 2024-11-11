"""
# **Cargo Deny**
A rule to run `cargo_deny` on a Rust workspace.
"""

def _impl_test(ctx):
    workspace = ctx.file.workspace.path
    cmd = [ctx.file._cargo_deny.short_path]
    cmd.append("--manifest-path {workspace}".format(workspace = workspace))
    cmd.append("check")
    cmd.append("--show-stats")
    cmd = " ".join(cmd)

    script = "exec {cmd}".format(cmd = cmd)

    ctx.actions.write(
        output = ctx.outputs.executable,
        content = script,
    )

    return [
        DefaultInfo(
            executable = ctx.outputs.executable,
            runfiles = ctx.runfiles(files = [ctx.file._cargo_deny, ctx.file._cargo] + ctx.files.srcs + ctx.files.workspace),
        ),
    ]

cargo_deny_test = rule(
    implementation = _impl_test,
    doc = """
    Test `cargo_deny check`.
    #### **Example**
    ```starlark
    cargo_deny_test(
        name = "my_cargo_deny_test",
        srcs = [":Cargo.toml"],
        workspace = ":Cargo.toml",
    )
    ```
    """,
    attrs = {
        "srcs": attr.label_list(
            allow_files = True,
            mandatory = True,
            doc = "Files you wish to include in the test.",
        ),
        "workspace": attr.label(
            mandatory = True,
            allow_single_file = True,
            doc = "The workspace directory as a file path where Cargo.toml is located.",
        ),
        "_cargo": attr.label(
            default = Label("@rules_rust//tools/upstream_wrapper:cargo"),
            allow_single_file = True,
            cfg = "exec",
            executable = True,
            doc = "The cargo executable to use.",
        ),
        "_cargo_deny": attr.label(
            default = Label("@multitool//tools/cargo_deny"),
            allow_single_file = True,
            cfg = "exec",
            executable = True,
            doc = "The cargo_deny executable to use.",
        ),
    },
    test = True,
)
