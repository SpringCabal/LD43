if not gadgetHandler:IsSyncedCode() then
	return
end

function gadget:GetInfo()
	return {
		name      = "Ork AI",
		desc      = "Handles ork AI",
		author    = "GoogleFrog",
		date      = "2 December 2018",
		license   = "GPL-v2",
		layer     = 0,
		enabled   = true
	}
end

local AIM_X = 5650
local AIM_Z = 5720
local AIM_VAR = 600
local AIM_REACHED_VAR = 2000
local ENEMY_TEAM = 1
local orksKilled = 0

local handledUnits = {
	[UnitDefNames["orksmall"].id] = true,
	[UnitDefNames["orkbig"].id] = true,
	[UnitDefNames["orkboss"].id] = true,
}

local IterableMap = VFS.Include("LuaRules/Gadgets/Include/IterableMap.lua")
local aiUnits = IterableMap.New()

--------------------------------------------------------------------------------
-- AI
--------------------------------------------------------------------------------

local function SetupAi(unitID)
	local dx = AIM_X + math.random()*AIM_VAR - AIM_VAR/2
	local dz = AIM_Z + math.random()*AIM_VAR - AIM_VAR/2
	local dy = Spring.GetGroundHeight(dx, dz)
	
	Spring.GiveOrderToUnit(unitID, CMD.FIGHT, {dx, dy, dz}, {})
end

local function CheckIdle(unitID)
	if Spring.GetCommandQueue(unitID, 0) ~= 0 then
		return
	end
	local dx = AIM_X + math.random()*AIM_REACHED_VAR - AIM_REACHED_VAR/2
	local dz = AIM_Z + math.random()*AIM_REACHED_VAR - AIM_REACHED_VAR/2
	local dy = Spring.GetGroundHeight(dx, dz)
	
	Spring.GiveOrderToUnit(unitID, CMD.FIGHT, {dx, dy, dz}, {})
end

local function SetupFlee(unitID, _, _, bossX, bossZ)
	local ux, _,uz = Spring.GetUnitPosition(unitID)
	local dx, dz = ux - bossX, uz - bossZ
	local dist = math.sqrt(dx*dx + dz*dz)
	
	local cx = ux + 4000*dx/dist
	local cz = uz + 4000*dz/dist
	local cy = ux + Spring.GetGroundHeight(cx, cz)
	
	Spring.Utilities.GiveClampedOrderToUnit(unitID, CMD.MOVE, {cx, cy, cz}, 0)
	Spring.GiveOrderToUnit(unitID, CMD.FIRESTATE_HOLDFIRE, {0, 0}, {})

end

--------------------------------------------------------------------------------
-- Event Handling
--------------------------------------------------------------------------------

function gadget:GameFrame(frame)
	local count = math.ceil(aiUnits.GetIndexMax()/30)
	for i = 1, count do
		CheckIdle(aiUnits.Next())
	end
end

function gadget:UnitCreated(unitID, unitDefID, teamID)
	if handledUnits[unitDefID] and teamID == ENEMY_TEAM then
		SetupAi(unitID)
		aiUnits.Add(unitID)
	end
end

function gadget:UnitDestroyed(unitID, unitDefID, teamID)
	if handledUnits[unitDefID] and teamID == ENEMY_TEAM then
		aiUnits.Remove(unitID)
		orksKilled = orksKilled + 1
		Spring.SetGameRulesParam("orksKilled", orksKilled)
	end
end

function GG.OrkAiFlee()
	
	local x = Spring.GetGameRulesParam("boss_x") or AIM_X
	local z = Spring.GetGameRulesParam("boss_z") or AIM_Z
	aiUnits.Apply(SetupFlee, x, z)
end

function gadget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		self:UnitCreated(unitID, Spring.GetUnitDefID(unitID), Spring.GetUnitTeam(unitID))
	end
end
