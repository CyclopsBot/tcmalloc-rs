# Incompatible flags

```
  --incompatible_disable_non_executable_java_binary # stardoc + rules_jvm_external seems like depriority
  --incompatible_stop_exporting_language_modules # rules_rust attempted fix but weird issue popped up
  --incompatible_enable_proto_toolchain_resolution # stardoc + buildifier
  --incompatible_disable_target_default_provider_fields # aspect bazel lib + rules_python + internal bazel rules
  --incompatible_auto_exec_groups # almost all aspect rules + rules_rust
```

# Not Planned

```
  --incompatible_no_rule_outputs_param # bazel built in memory starlark not recommended to be enabledby bazel team https://github.com/bazelbuild/bazel/issues/7977#issuecomment-1246684425
  --incompatible_check_visibility_for_toolchains # basically every rule cant even find bazel issue mentioning the flag
```
