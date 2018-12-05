if not gadgetHandler:IsSyncedCode() then
	return
end

function gadget:GetInfo()
	return {
		name      = "storyteller",
		desc      = "",
		author    = "gajop",
		date      = "LD43",
		license   = "GPL-v2",
		layer     = 0,
		enabled   = true
	}
end

local LOG_SECTION = "Storyteller"

local GAME_FRAME_PER_SEC = 33
local ourTeam = 0
local enemyTeam = 1
local gaiaTeam = Spring.GetGaiaTeamID()

local story
local s11n
local lastStepFrame
local waitFrames
local spawnAreas

local DEV_NO_RUN = false
local NEVER_RUN = false

-- ugh
local killed = {}
function gadget:Initialize()
	Spring.Log(LOG_SECTION, LOG.NOTICE, "Initializing storyteller...")
	spawnAreas = loadSpawnAreas()
	lastStepFrame = -math.huge
	waitFrames = 0
	story = GetStory()
	s11n = GG.s11n:GetUnitBridge()

	for _, unitID in pairs(Spring.GetAllUnits()) do
		-- Spring.DestroyUnit(unitID, false, true)
		killed[unitID] = true
	end

	if Spring.GetGameRulesParam("sb_gameMode") == nil then
        Spring.RevertHeightMap(0, 0, Game.mapSizeX, Game.mapSizeZ, 1)
    end

	Spring.SetGameRulesParam("introEvent", "")
end

function GetStory()
	return {
		-- {
		--     name = "intro",
		-- 	about = "Some introductory text.",
		-- 	time = 5,
		-- },
		{ -- Night 1
			name = "spawn",
			humanName = "First Night",
			units = {
				orksmall = 40,
				orkbig = 1,
			},
			team = enemyTeam,
			time = 1,
		},
		{ -- Night 1
			name = "spawn",
			humanName = "First Night b",
			units = {
				orksmall = 30,
				orkbig = 2,
			},
			team = enemyTeam,
			time = 10,
		},
		{  -- Night 2
			name = "spawn",
			humanName = "Second Night",
			units = {
				orksmall = 70,
				orkbig = 3,
			},
			team = enemyTeam,
			time = 25,
		},
		{ -- Night 3
			name = "spawn",
			humanName = "Getting Bigger",
			units = {
				orksmall = 70,
				orkbig = 8,
			},
			team = enemyTeam,
			time = 12,
		},
		{  -- Night 4
			name = "spawn",
			humanName = "Quick Followup",
			units = {
				orksmall = 70,
				orkbig = 6,
			},
			team = enemyTeam,
			time = 25,
		},
		{  -- Night 5
			name = "spawn",
			humanName = "Big Ork Wave",
			units = {
				orksmall = 32,
				orkbig = 28,
			},
			team = enemyTeam,
			time = 25,
		},
		{  -- Night 6
			name = "spawn",
			humanName = "The Swarm",
			units = {
				orksmall = 95,
				orkbig = 3,
			},
			team = enemyTeam,
			time = 8,
		},
		{  -- Night 6a
			name = "spawn",
			humanName = "The Swarm",
			units = {
				orksmall = 100,
				orkbig = 2,
			},
			team = enemyTeam,
			time = 35,
		},
		{  --  More Nights
			name = "spawn",
			humanName = "More Nights",
			units = {
				orksmall = 70,
				orkbig = 10,
			},
			team = enemyTeam,
			time = 20,
		},
		{  -- Night 7
			name = "spawn",
			humanName = "Final Boss",
			ableToWin = true,
			units = {
				orksmall = 90,
				orkbig = 8,
				orkboss = 1,
			},
			team = enemyTeam,
			time = 30,
		},
		{  -- Forever spawn
			name = "spawn",
			infiniteLoop = true,
			humanName = "The Loop",
			units = {
				orksmall = 35,
				orkbig = 3,
			},
			team = enemyTeam,
			time = 20,
		},
	}
end

local function findPosition(defName)
	--local area = spawnAreas[math.random(#spawnAreas)]
	--local x = math.random(area.minx, area.maxx)
	--local z = math.random(area.minz, area.maxz)
	local cX = 5700
	local cZ = 5850
	local radius = 3200
	local angle = math.random()*320
	if angle > 65 and angle < 85 then
		angle = angle + 320 - 50
	end
	if angle > 180 and angle < 200 then
		angle = angle + 340 - 130
	end
	
	angle = angle*math.pi/180
	local x = cX + math.cos(angle)*radius
	local z = cZ - math.sin(angle)*radius
	
	local y = Spring.GetGroundHeight(x, z)
	return x, y, z
end

function SpawnUnits(units, team)
	for defName, count in pairs(units) do
		local obj = {
			defName = defName,
			team = team,
		}
		if todoIsDecal then -- TODO
			obj.team = gaiaTeam
			obj.neutral = true
			obj.blocking = {
				blockEnemyPushing = true,
				blockHeightChanges = false,
				crushable = false,
				isBlocking = true,
				isProjectileCollidable = false,
				isRaySegmentCollidable = false,
				isSolidObjectCollidable = true,
			}
		end

		for i = 1, count do
			local x, y, z = findPosition(defName)
			obj.pos = {
				x = x,
				y = y,
				z = z,
			}
			s11n:Add(obj)
		end
	end
end

function DoStep(step)
	if step.name == "spawn" then
		Spring.Echo("spawn", step.humanName)
		SpawnUnits(step.units, step.team)
	end
	if step.ableToWin then
		Spring.SetGameRulesParam("gameEnd", "victoryPossible")
	end
end

function NextStep()
	Spring.Log(LOG_SECTION, LOG.NOTICE, "Next step")
	local step = story[1]
	DoStep(step)
	if not step.infiniteLoop then
		table.remove(story, 1)
	end
	if step.time then
		waitFrames = step.time * GAME_FRAME_PER_SEC
	elseif step.name == "spawn" then
		waitFrames = 30 * GAME_FRAME_PER_SEC
	else
		waitFrames = 10 * GAME_FRAME_PER_SEC
	end
end

local function IsNextStepTime()
	if #story == 0 then
		return false
	end
	local now = Spring.GetGameFrame()
	if not (now - lastStepFrame >= waitFrames) then
		return false
	end

	lastStepFrame = now
	return true
end

function loadSpawnAreas()
	local areas = loadstring(VFS.LoadFile("LuaRules/Configs/spawn_points.lua"))()
	local parsedAreas = {}
	for _, area in pairs(areas) do
		table.insert(parsedAreas, {
			minx = area.pos.x - area.size.x / 2,
			maxx = area.pos.x + area.size.x / 2,
			minz = area.pos.z - area.size.z / 2,
			maxz = area.pos.z + area.size.z / 2,
		})
	end
	return parsedAreas
end

local lastGameMode = nil
function gadget:GameFrame()
	if Spring.GetGameRulesParam("gameEnd") == "victory" then
		return
	end
	if NEVER_RUN then
		return
	end
	if DEV_NO_RUN then
		local currentGameMode = Spring.GetGameRulesParam("gameMode")
		if lastGameMode ~= currentGameMode then
			lastGameMode = currentGameMode
		
			if currentGameMode ~= "develop" then
				self:Initialize()
			end
			return
		end
		if currentGameMode == "develop" then
			return
		end
	end
	local frame = Spring.GetGameFrame()
	if IsNextStepTime() then
		NextStep()
	end
end

GG.NextStep = NextStep
