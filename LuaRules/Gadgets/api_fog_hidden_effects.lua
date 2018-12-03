function gadget:GetInfo() return {
	name      = "Fog Hidden Effects API",
	desc      = "API for playing sounds only for players with vision",
	author    = "Google Frog",
	date      = "17 Jan 2016",
	license   = "GNU GPL, v2 or later",
	layer     = 0,
	enabled   = true,
} end

if gadgetHandler:IsSyncedCode() then
	local SendToUnsync = SendToUnsynced

	function GG.PlaySound(sound, volume, x, y, z)
		SendToUnsync("playSound", sound, volume, x, y, z)
	end
else
	local spPlaySoundFile = Spring.PlaySoundFile

	local function playSound(_, sound, volume, x, y, z)
		spPlaySoundFile(sound, volume, x, y, z)
	end

	function gadget:Initialize()
		gadgetHandler:AddSyncAction("playSound", playSound)
	end
end
