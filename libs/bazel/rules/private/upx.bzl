"""
# **UPX**
A rule to run `upx` on a binary.
"""

def _resource_set(_, __):
    return {"cpu": 4, "memory": 2096}

def _impl(ctx):
    inputs = ctx.files.srcs[:]
    out = ctx.outputs.out or ctx.actions.declare_file(ctx.attr.name)
    args = ctx.actions.args()
    args.add("--best")
    args.add("--lzma")
    args.add("-o", out)
    args.add_all(inputs)

    ctx.actions.run(
        executable = ctx.executable._upx,
        inputs = depset(direct = inputs, transitive = [
            src[DefaultInfo].default_runfiles.files
            for src in ctx.attr.srcs
        ]),
        outputs = [out],
        progress_message = "Compressing %{input}",
        arguments = [args],
        mnemonic = "UpxPack",
        execution_requirements = {
            "supports-path-mapping": "1",
        },
        resource_set = _resource_set,
    )

    return DefaultInfo(files = depset([out]), runfiles = ctx.runfiles([out]))

upx = rule(
    implementation = _impl,
    doc = """
    Compresses a set of binaries into upx packed ones.
    #### **Example**
    ```starlark
    upx(
        name = "my_upx_binary_target",
        srcs = [":binary_target"],
        out = "binary_name", # optional
    )
    ```
    """,
    attrs = {
        "out": attr.output(
            doc = "Resulting file to write. If absent, `[name]` is written.",
        ),
        "srcs": attr.label_list(
            allow_files = True,
            doc = "Files you wish to compress.",
        ),
        "_upx": attr.label(
            default = Label("@multitool//tools/upx"),
            allow_single_file = True,
            cfg = "exec",
            executable = True,
            doc = "The upx executable to use.",
        ),
    },
)
