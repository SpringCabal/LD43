--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
function gadget:GetInfo()
	return {
		name      = "Setup Modrules",
		desc      = "Does things which could be modrules.",
		author    = "GoogleFrog",
		date      = "2 December 2018",
		license   = "GNU GPL, v2 or later",
		layer     = 10,
		enabled   = true
	}
end

function gadget:Initialize()
	Spring.SetGlobalLos(0, true)
	Spring.SetGlobalLos(1, true)
	Spring.SetGlobalLos(2, true)
end