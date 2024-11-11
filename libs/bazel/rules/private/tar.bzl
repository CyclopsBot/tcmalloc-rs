"""
# **Tar**
Rules for interacting with tars.
"""

load("@aspect_bazel_lib//lib:tar.bzl", "mtree_mutate", "mtree_spec", _aspect_tar = "tar")

def _impl(name, srcs, owner, awk_script, compress, visibility, tags):
    mtree_spec(
        name = name + "_mtree",
        srcs = srcs,
    )
    mtree_mutate(
        name = name + "_mutate",
        mtree = ":{}_mtree".format(name),
        strip_prefix = native.package_name(),
        owner = owner,
        awk_script = awk_script,
    )
    _aspect_tar(
        name = name,
        srcs = srcs,
        mtree = ":{}_mutate".format(name),
        compress = compress,
        visibility = visibility,
        tags = tags,
    )

tar = macro(
    implementation = _impl,
    doc = """
    Creates a hermetic tarball.

    #### **Example**
    ```starlark
    tar(
        name = "my_tarball",
        srcs = ["file1", "file2"],
        owner = 10000, # optional
        awk_script = "//tools:custom_mtree.awk", # optional
        compress = "zstd", # optional
    )
    ```
    """,
    attrs = {
        "awk_script": attr.label(
            configurable = False,
            default = "//tools:modify_mtree.awk",
            doc = "Awk script to modify the mtree of the tar archive.",
        ),
        "compress": attr.string(
            configurable = False,
            default = "zstd",
            doc = "Compression algorithm to use.",
        ),
        "owner": attr.int(
            configurable = False,
            default = 10000,
            doc = "Owner UID for files in the tar archive.",
        ),
        "srcs": attr.label_list(
            mandatory = True,
            doc = "Files you wish to include in the image.",
        ),
        "tags": attr.string_list(
            configurable = False,
        ),
    },
)
