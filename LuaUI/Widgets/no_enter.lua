function widget:GetInfo()
	return {
		name      = "No Enter",
		desc      = "",
		author    = "GoogleFrog",
		date      = "in the future",
		license   = "GPL-v2",
		layer     = 10000,
		enabled   = true,
	}
end

include('keysym.h.lua')
local RETURN = KEYSYMS.RETURN
function widget:KeyPress(key, mods, isRepeat)
    if key == RETURN then -- doesn't work, idk
        return true
    end
end
