<!-- Generated with Stardoc: http://skydoc.bazel.build -->

# **Transition**
Utilities for transitioning targets/filegroups in bazel.

<a id="multi_arch"></a>

## multi_arch

<pre>
load("@//libs/bazel/rules/private:transition.bzl", "multi_arch")

multi_arch(<a href="#multi_arch-name">name</a>, <a href="#multi_arch-image">image</a>, <a href="#multi_arch-platforms">platforms</a>)
</pre>

Transition an OCI image to support multiple architectures.
#### **Example**
```starlark
multi_arch(
    name = "my_multi_arch_image",
    image = "//path/to/image",
    platforms = ["//tools/platforms:linux_amd64", "//tools/platforms:linux_arm64"],
)
```

**ATTRIBUTES**


| Name  | Description | Type | Mandatory | Default |
| :------------- | :------------- | :------------- | :------------- | :------------- |
| <a id="multi_arch-name"></a>name |  A unique name for this target.   | <a href="https://bazel.build/concepts/labels#target-names">Name</a> | required |  |
| <a id="multi_arch-image"></a>image |  Oci image to transition.   | <a href="https://bazel.build/concepts/labels">Label</a> | optional |  `None`  |
| <a id="multi_arch-platforms"></a>platforms |  The platforms you wish to transition   | <a href="https://bazel.build/concepts/labels">List of labels</a> | optional |  `[]`  |


