require "Koru"
local MajorMode = require "Koru.MajorMode"

LogViewMode = {}
LogViewMode.__index = LogViewMode

function LogViewMode:new()
    local obj = MajorMode()
    obj = MajorMode.extend(obj, LogViewMode)
    return obj
end

set_major_mode("**Warnings**", LogViewMode:new())
set_major_mode("**Errors**", LogViewMode:new())