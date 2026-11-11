# mise plugins for the checks

`kubaos-test-tool/` is the test Suite's mise tool (`crates/kubaos-core/src/suites/catalog.rs`, `TEST`): an asdf plugin that installs a one-line `kubaos-test-tool` printing its version, with no network. The harness links it with `mise plugins link kubaos-test-tool test/mise/kubaos-test-tool` before `kubaos suite add test`, so `mise install kubaos-test-tool@1` finds it. It offers 1.0.0, 1.0.1, 2.0.0 and 3.0.0, and fails to install the version `KUBAOS_TEST_TOOL_FAIL` names.
