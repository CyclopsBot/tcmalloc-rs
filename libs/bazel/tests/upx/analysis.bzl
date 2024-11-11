""""""

load("@rules_testing//lib:analysis_test.bzl", "analysis_test", "test_suite")
load("//libs/bazel/rules:defs.bzl", "upx")

def _test_analysis(name):
    upx(name = name + "_upx", srcs = ["bin"])
    analysis_test(
        name = name,
        impl = _test_analysis_impl,
        target = name + "_upx",
    )

def _test_analysis_impl(env, target):
    env.expect.that_target(target).default_outputs().contains(
        "libs/bazel/tests/upx/test_analysis_upx",
    )

def upx_test_suite(name):
    test_suite(
        name = name,
        tests = [
            _test_analysis,
        ],
    )
