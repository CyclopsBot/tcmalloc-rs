""""""

load("@rules_testing//lib:analysis_test.bzl", "analysis_test", "test_suite")
load("//libs/bazel/rules:defs.bzl", "tar")

def _test_analysis(name):
    tar(name = name + "_tar", srcs = ["test.txt"])
    analysis_test(
        name = name,
        impl = _test_analysis_impl,
        target = name + "_tar",
    )

def _test_analysis_impl(env, target):
    env.expect.that_target(target).default_outputs().contains(
        "libs/bazel/tests/tar/test_analysis_tar.tar.zst",
    )

def tar_test_suite(name):
    test_suite(
        name = name,
        tests = [
            _test_analysis,
        ],
    )
