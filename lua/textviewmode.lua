local Koru = require "Koru"
local MajorMode = require "Koru.MajorMode"

TextViewMode = {}
function TextViewMode:new()
    TextViewMode.__index = TextViewMode
    setmetatable(TextViewMode, { __index = MajorMode })
    local obj = MajorMode()
    setmetatable(obj, TextViewMode)
    obj.line_number = true
    return obj
end

function TextViewMode:set_line_number(enable)
    self.line_number = enable
end


local function file_open_hook(file_index, file_ext)
    set_major_mode(file_index, TextViewMode:new())
    print("hook!")
end

add_file_open_hook("TextViewMode", file_open_hook)