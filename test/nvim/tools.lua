-- For the `nvim` end-to-end check (`test/lib/e2e.sh`), run from the repo's root:
-- what Mason would install, and for each npm-based tool of the shared extras,
-- the program it runs. A server reports once it attaches to a buffer of its
-- kind, and markdownlint-cli2 once it flags a file that breaks a rule. One
-- line each: `MASON <packages>`, `SERVER <name> <mason> <program>`,
-- `TOOL <name> <program>`, `LINT <name> <diagnostics>`.
local function report(...)
  io.write(table.concat({ ... }, " "), "\n")
end

-- A server's client on the current buffer, waiting up to 30 s for it to start.
local function attached(name)
  local client
  vim.wait(30000, function()
    client = vim.lsp.get_clients({ bufnr = 0, name = name })[1]
    return client ~= nil
  end, 100)
  return client
end

report("MASON", table.concat(LazyVim.opts("mason.nvim").ensure_installed, ","))

local servers = LazyVim.opts("nvim-lspconfig").servers
-- Each server's program by name: jsonls and yamlls build their command in a
-- function, which runs the project's `node_modules/.bin` copy when there is one
-- and the PATH one otherwise.
local buffers = {
  { file = "config/nvim/lazy-lock.json", server = "jsonls", program = "vscode-json-language-server" },
  { file = ".github/workflows/check.yml", server = "yamlls", program = "yaml-language-server" },
  { file = "test/Dockerfile", server = "dockerls", program = "docker-langserver" },
  -- Neovim gives compose.yaml plain `yaml`; the compose server asks for this.
  {
    file = "test/compose.yaml",
    server = "docker_compose_language_service",
    program = "docker-compose-langserver",
    filetype = "yaml.docker-compose",
  },
  { file = "scripts/dot-theme", server = "bashls", program = "bash-language-server" },
}
for _, buffer in ipairs(buffers) do
  vim.cmd.edit(buffer.file)
  if buffer.filetype then
    vim.bo.filetype = buffer.filetype
  end
  local client = attached(buffer.server)
  report(
    "SERVER",
    buffer.server,
    tostring(servers[buffer.server].mason),
    client and vim.fn.exepath(buffer.program) or "none"
  )
end

for _, tool in ipairs({ "prettier", "markdownlint-cli2", "markdown-toc" }) do
  report("TOOL", tool, vim.fn.exepath(tool))
end

-- Three blank lines in a row break markdownlint's MD012.
local markdown = vim.fn.tempname() .. ".md"
vim.fn.writefile({ "# Probe", "", "", "", "Text." }, markdown)
vim.cmd.edit(markdown)
local namespace = require("lint").get_namespace("markdownlint-cli2")
local found = {}
require("lint").try_lint()
vim.wait(30000, function()
  found = vim.diagnostic.get(0, { namespace = namespace })
  return #found > 0
end, 500)
report("LINT", "markdownlint-cli2", tostring(#found))
