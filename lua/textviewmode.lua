local Koru = require "Koru"
local MajorMode = require "Koru.MajorMode"
local StyledText = require "Koru.StyledText"
local StyledText = require "Koru.StyledText.StyledText"

TextViewMode = {}
TextViewMode.__index = TextViewMode
function TextViewMode:new()
    local obj = MajorMode()
    obj = MajorMode.extend(obj, TextViewMode)
    obj.line_number = true
    return obj
end

function TextViewMode:set_line_number(enable)
    self.line_number = enable
end

local function num_digits(n)
    if n == 0 then
        return 1
    end

    return math.floor(math.log(n, 10)) + 1
end

function TextViewMode:modify_line(styled_file, total_lines)
    if not self.line_number then
        return
    end

    local line_count = total_lines
    local digit_count = num_digits(total_lines)
    
    for i = 0, line_count do
        styled_file:prepend_segment(i, StyledText(string.format("%" .. tostring(digit_count) .. "d|", i + 1)))
    end

    return styled_file

end


local function file_open_hook(file_name, file_ext)
    set_major_mode(file_name, TextViewMode:new())
    print("hook!")
end

add_file_open_hook("TextViewMode", file_open_hook)