--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name      = "Game End",
		desc      = "Handles team/allyteam deaths and declares gameover",
		author    = "Andrea Piras",
		date      = "June, 2013",
		license   = "GNU GPL, v2 or later",
		layer     = 0,
		enabled   = true  --  loaded by default?
	}
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

--SYNCED
if (not gadgetHandler:IsSyncedCode()) then
   return false
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local orksmall = UnitDefNames["orksmall"].id
local orkbig = UnitDefNames["orkbig"].id
local orkboss = UnitDefNames["orkboss"].id
local defs = {orkboss}
local ENEMY_TEAM = 1

local initializeFrame = 0

function gadget:Initialize()
	initializeFrame = Spring.GetGameFrame() or 0
	Spring.SetGameRulesParam("gameEnd", "start")
end

function gadget:GameFrame(frame)
	if Spring.IsGameOver() then
		return
	end
	if frame > initializeFrame + 2 and Spring.GetGameRulesParam("gameEnd") == "victoryPossible" then
		for _, defID in pairs(defs) do
			if Spring.GetTeamUnitDefCount(ENEMY_TEAM, defID) > 0 then
				return
			end
		end
		-- count orks
		Spring.SetGameRulesParam("gameEnd", "victory")
		GG.OrkAiFlee()
	end
end
