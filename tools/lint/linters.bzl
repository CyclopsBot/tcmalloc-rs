"Define linter aspects"

load("@aspect_rules_lint//lint:buf.bzl", "lint_buf_aspect")
load("@aspect_rules_lint//lint:lint_test.bzl", "lint_test")
load("@aspect_rules_lint//lint:shellcheck.bzl", "lint_shellcheck_aspect")

shellcheck = lint_shellcheck_aspect(
    binary = "@multitool//tools/shellcheck",
    config = "@@//tools/lint:.shellcheckrc",
)
shellcheck_test = lint_test(aspect = shellcheck)

buf = lint_buf_aspect(
    config = "@@//tools/lint:buf.yaml",
)
buf_test = lint_test(aspect = buf)
