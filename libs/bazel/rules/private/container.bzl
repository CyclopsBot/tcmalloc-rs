"""
# **Container**
Utilities for building containers.
"""

load("@aspect_bazel_lib//lib:tar.bzl", "mtree_mutate", "mtree_spec", _aspect_tar = "tar")
load("@rules_oci//oci:defs.bzl", "oci_image", "oci_image_index", "oci_load", "oci_push")
load("//libs/bazel/rules/private:transition.bzl", "multi_arch")
load("//libs/bazel/rules/private:upx.bzl", "upx")

def build_image(
        name,
        srcs,
        repository,
        binary_name = "bin",
        remote_tag = "latest",
        base = "//infra/images/base:image",
        platforms = ["//tools/platforms:linux_aarch64", "//tools/platforms:linux_amd64"]):
    """
    Builds a multi-architecture OCI image and index.

    #### **flow**
    ```mermaid
        graph TD;
            binary-->upx;
            upx-->tar;
            tar-->oci_image;
            oci_image-->march_image;
            march_image-->image_index;
            image_index-->target.push;
            image_index-->target.sha256;
            oci_image-->target.load;
    ```

    #### **Example**
    ```starlark
    build_image(
        name = "my_multi_arch_image",
        base = "//infra/images/base:image",
        srcs = ["file1", "file2"],
        platforms = ["//tools/platforms:linux_amd64_musl"], # optional
        entry_point = "/my/entrypoint", # optional
    )
    ```

    Args:
        name: (String). Name of the target.
        srcs: (String). Files you wish to include in the image.
        repository: (String). The remote repository you want to use.
        binary_name: (String). Name of the binary that the container will run.
        remote_tag: (String). What to tag the remote image with ex :latest.
        base: (Label). The base image to use.
        platforms: (Label). Bazel platform you wish to use.
    """
    repo_tags = "{repository}:{remote_tag}".format(repository = repository, remote_tag = remote_tag)
    entrypoint = "/{}.upx".format(binary_name)
    upx(
        name = "{}.upx".format(name),
        srcs = srcs,
        out = "{}.upx".format(binary_name),
    )
    mtree_spec(
        name = "_{}.mtree".format(name),
        srcs = [":{}.upx".format(name)],
    )
    mtree_mutate(
        name = "_{}.mutate".format(name),
        mtree = ":_{}.mtree".format(name),
        strip_prefix = native.package_name(),
        owner = 10000,
        awk_script = "//tools:modify_mtree.awk",
    )
    _aspect_tar(
        name = "_{}.tar".format(name),
        srcs = [":{}.upx".format(name)],
        mtree = ":_{}.mutate".format(name),
        compress = "zstd",
    )
    oci_image(
        name = "_{}.image".format(name),
        base = base,
        tars = [":_{}.tar".format(name)],
        entrypoint = [entrypoint],
        tags = ["local"],
    )
    oci_load(
        name = "{}.load".format(name),
        image = ":_{}.image".format(name),
        repo_tags = [repo_tags],
    )
    multi_arch(
        name = "_{}.march".format(name),
        image = ":_{}.image".format(name),
        platforms = platforms,
    )
    oci_image_index(
        name = name,
        images = [
            ":_{}.march".format(name),
        ],
    )
    oci_push(
        name = "{}.push".format(name),
        image = ":_{}.image".format(name),
        remote_tags = [remote_tag],
        repository = repository,
    )
    native.alias(
        name = "{}.sha256".format(name),
        actual = "_{}.image.digest".format(name),
    )
