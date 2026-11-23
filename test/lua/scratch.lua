-- A test file's scratch folder under the system's temp dir, holding every home
-- and file the file's tests make.
--
--   local scratch <close> = require("scratch").new()
--   local home = scratch:path()
--
-- Declared `<close>` in the file's main chunk, the folder is removed when the
-- chunk ends, when an error escapes it, and at `os.exit(code, true)`: a plain
-- `os.exit(code)` skips the close and leaves the folder behind.

local Scratch = {}
Scratch.__index = Scratch

-- `path` single-quoted for the shell, a quote in it closed, escaped and reopened.
local function quoted(path)
  return "'" .. path:gsub("'", "'\\''") .. "'"
end

-- Write permission comes back first: `rm -rf` can't empty a folder made read-only. The removal
-- runs whatever the chmod says, so one entry it can't change doesn't keep the rest.
function Scratch:__close()
  local root = quoted(self.root)
  os.execute("chmod -R u+w " .. root .. "; rm -rf " .. root)
end

function Scratch.new()
  local mktemp = assert(io.popen("mktemp -d -t kubaos-lua-test.XXXXXX"))
  local root = mktemp:read("l")
  mktemp:close()
  assert(root and root ~= "", "mktemp made no scratch folder")
  return setmetatable({ root = root, count = 0 }, Scratch)
end

-- A fresh path inside the folder, with nothing made there yet.
function Scratch:path()
  self.count = self.count + 1
  return self.root .. "/" .. self.count
end

return Scratch
