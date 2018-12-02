function widget:GetInfo()
    return {
        name      = "day_night mechanics",
        author    = "gajop",
        date      = "LD43",
        license   = "GNU GPL, v2 or later",
        layer     = 0,
        enabled   = true,
    }
end

local isNight = true
local nightSetup
function widget:Initialize()
    nightSetup = loadstring(VFS.LoadFile("LuaUI/Configs/day_night.lua"))().night
    if isNight then
        Spring.SetSunLighting(nightSetup)
    end
end