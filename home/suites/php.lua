-- The PHP Suite's Neovim settings, part of the Defaults while the Suite is
-- installed (home/suites/php.nix): LazyVim's PHP extra, run on the Suite's
-- phpactor, php-cs-fixer, phpcs and Xdebug's debug adapter.
-- https://github.com/xdebug/vscode-php-debug#vs-code-configuration
local tools = require("config.suite_tools")

return {
  { import = "lazyvim.plugins.extras.lang.php" },
  tools.on_path({
    servers = { "phpactor" },
    packages = { "phpcs", "php-cs-fixer" },
  }),
  {
    "mfussenegger/nvim-dap",
    optional = true,
    opts = function()
      -- The extra gives the adapter only; these are the adapter's own two
      -- starting points. Xdebug's client port is 9003 unless php.ini moves it.
      local dap = require("dap")
      dap.configurations.php = dap.configurations.php
        or {
          {
            type = "php",
            request = "launch",
            name = "Listen for Xdebug",
            port = 9003,
          },
          {
            type = "php",
            request = "launch",
            name = "Launch current script",
            program = "${file}",
            cwd = "${fileDirname}",
            -- The adapter picks a free port and fills in `${port}` itself.
            port = 0,
            runtimeArgs = { "-dxdebug.start_with_request=yes" },
            env = { XDEBUG_MODE = "debug,develop", XDEBUG_CONFIG = "client_port=${port}" },
          },
        }
    end,
  },
}
