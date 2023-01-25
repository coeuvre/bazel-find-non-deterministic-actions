# Find non-deterministic actions in Bazel

Given two or more execution logs produced by `--execution_log_json_file` from different Bazel invocations, the tool will find and print non-deterministic actions if any.

For example:

```
$ cargo run --release -- --execution_log_json_file=a.json --execution_log_json_file=b.json

...
Outputs of the same action `Executing genrule //tools/osx:xcode-locator-genrule` are different:
--- original
+++ modified
@@ -42,7 +42,7 @@
     {
       "path": "bazel-out/darwin-opt/bin/tools/osx/xcode-locator",
       "digest": {
-        "hash": "f93c577f5e57dd30cc5709302ec8b1ab4eb5cc0c8943a35f521b28ef0d5d1053",
+        "hash": "f8ed1308ff7e88edbd8c656426ed2db58eed64d5d931bc6298e4e04786607841",
         "sizeBytes": "188768",
         "hashFunctionName": "SHA-256"
       }

```