
if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name      = "Status Effects",
		desc      = "Handles timed status effects.",
		author    = "GoogleFrog",
		date      = "3 December 2018",
		license   = "GNU GPL, v2 or later",
		layer     = 0,
		enabled   = true,
	}
end

local IterableMap = VFS.Include("LuaRules/Gadgets/Include/IterableMap.lua")

local PLAYER_TEAM = 0
local units = IterableMap.New()

local StatusEffects = {}

function StatusEffects.Adrenaline(unitID, endFrame)
	local data = units.Get(unitID) or {}
	
	GG.Attributes.AddEffect(unitID, "buff", {move = 2, reload = 2})
	
	data.adrenalineEnd = endFrame
	units.Set(unitID, data) -- Also adds
end

local function CheckUnit(unitID, data, frame)
	if data.adrenalineEnd and (frame > data.adrenalineEnd) then
		GG.Attributes.RemoveEffect(unitID, "buff")
		data.adrenalineEnd = nil
	end
	
	if not (data.adrenalineEnd or data.stunEnd) then
		units.Remove(unitID)
	end
end

function gadget:GameFrame(frame)
	local count = math.ceil(units.GetIndexMax()/30)
	for i = 1, count do
		local unitID, data = units.Next()
		CheckUnit(unitID, data, frame)
	end
end

function gadget:UnitDestroyed(unitID)
	units.Remove(unitID)
end

function gadget:UnitCreated(unitID, unitDefID, teamID)
	if teamID == PLAYER_TEAM then
		GG.Attributes.AddEffect(unitID, "halfSpeed", {move = 0.5})
	end
end

function gadget:Initialize()
	GG.StatusEffects = StatusEffects
	
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		self:UnitCreated(unitID, Spring.GetUnitDefID(unitID), Spring.GetUnitTeam(unitID))
	end
end