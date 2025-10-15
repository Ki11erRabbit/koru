require "Koru"
local MajorMode = require "Koru.MajorMode"

LogViewMode = {}
LogViewMode.__index = MajorMode
setmetatable(LogViewMode, { __index = MajorMode })
function LogViewMode:new()
    local obj = MajorMode()
    setmetatable(obj, LogViewMode)
    return obj
end

set_major_mode("**Warnings**", LogViewMode:new())
set_major_mode("**Errors**", LogViewMode:new())