<!-- Generated with Stardoc: http://skydoc.bazel.build -->

# **Cargo Deny**
A rule to run `cargo_deny` on a Rust workspace.

<a id="cargo_deny_test"></a>

## cargo_deny_test

<pre>
load("@//libs/bazel/rules/private:cargo_deny.bzl", "cargo_deny_test")

cargo_deny_test(<a href="#cargo_deny_test-name">name</a>, <a href="#cargo_deny_test-srcs">srcs</a>, <a href="#cargo_deny_test-workspace">workspace</a>)
</pre>

Test `cargo_deny check`.
#### **Example**
```starlark
cargo_deny_test(
    name = "my_cargo_deny_test",
    srcs = [":Cargo.toml"],
    workspace = ":Cargo.toml",
)
```

**ATTRIBUTES**


| Name  | Description | Type | Mandatory | Default |
| :------------- | :------------- | :------------- | :------------- | :------------- |
| <a id="cargo_deny_test-name"></a>name |  A unique name for this target.   | <a href="https://bazel.build/concepts/labels#target-names">Name</a> | required |  |
| <a id="cargo_deny_test-srcs"></a>srcs |  Files you wish to include in the test.   | <a href="https://bazel.build/concepts/labels">List of labels</a> | required |  |
| <a id="cargo_deny_test-workspace"></a>workspace |  The workspace directory as a file path where Cargo.toml is located.   | <a href="https://bazel.build/concepts/labels">Label</a> | required |  |


