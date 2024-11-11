"""
# **Transition**
Utilities for transitioning targets/filegroups in bazel.
"""

# buildifier: disable=unused-variable
def _multiarch_transition(settings, attr):
    return [
        {"//command_line_option:platforms": str(platform)}
        for platform in attr.platforms
    ]

multiarch_transition = transition(
    implementation = _multiarch_transition,
    inputs = [],
    outputs = ["//command_line_option:platforms"],
)

def _impl(ctx):
    return DefaultInfo(files = depset(ctx.files.image))

multi_arch = rule(
    implementation = _impl,
    doc = """
    Transition an OCI image to support multiple architectures.
    #### **Example**
    ```starlark
    multi_arch(
        name = "my_multi_arch_image",
        image = "//path/to/image",
        platforms = ["//tools/platforms:linux_amd64", "//tools/platforms:linux_arm64"],
    )
    ```
    """,
    attrs = {
        "image": attr.label(cfg = multiarch_transition, doc = "Oci image to transition."),
        "platforms": attr.label_list(doc = "The platforms you wish to transition"),
        "_allowlist_function_transition": attr.label(
            default = "@bazel_tools//tools/allowlists/function_transition_allowlist",
        ),
    },
)
