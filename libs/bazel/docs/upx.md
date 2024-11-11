<!-- Generated with Stardoc: http://skydoc.bazel.build -->

# **UPX**
A rule to run `upx` on a binary.

<a id="upx"></a>

## upx

<pre>
load("@//libs/bazel/rules/private:upx.bzl", "upx")

upx(<a href="#upx-name">name</a>, <a href="#upx-srcs">srcs</a>, <a href="#upx-out">out</a>)
</pre>

Compresses a set of binaries into upx packed ones.
#### **Example**
```starlark
upx(
    name = "my_upx_binary_target",
    srcs = [":binary_target"],
    out = "binary_name", # optional
)
```

**ATTRIBUTES**


| Name  | Description | Type | Mandatory | Default |
| :------------- | :------------- | :------------- | :------------- | :------------- |
| <a id="upx-name"></a>name |  A unique name for this target.   | <a href="https://bazel.build/concepts/labels#target-names">Name</a> | required |  |
| <a id="upx-srcs"></a>srcs |  Files you wish to compress.   | <a href="https://bazel.build/concepts/labels">List of labels</a> | optional |  `[]`  |
| <a id="upx-out"></a>out |  Resulting file to write. If absent, `[name]` is written.   | <a href="https://bazel.build/concepts/labels">Label</a> | optional |  `None`  |


