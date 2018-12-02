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

	Spring.SetGameRulesParam("introEvent", "")
end

function GetStory()
	-- Spawn a few pirates
	-- Spawn a drilling boat
	-- Introduce food and healing
	-- Spawn pirates and a drilling boat
	-- Introduce selling
	-- Spawn stuff
	-- Introduce heat
	-- Spawn stuff
	-- Introduce global warming ("It's getting warmer")
	-- Spawn stuff
	return {
		-- {
		--     name = "intro",
		-- 	about = "Some introductory text.",
		-- 	time = 5,
		-- },
		{ -- Night 1
			name = "spawn",
			units = {
				orksmall = 15
			},
			team = enemyTeam,
			time = 1,
		},
		{
			name = "intro",
			about = "{Wave has spawned text.}",
			time = 5,
		},
		{  -- Night 2
			name = "spawn",
			units = {
				orksmall = 15,
				orkbig = 3,
			},
			team = enemyTeam,
			time = 10,
		},
		{
			name = "intro",
			about = "Some more info.",
			time = 20,
		},
		{
			name = "intro",
			about = "Another wave...",
			time = 1,
		},
		{ -- Night 3
			name = "spawn",
			units = {
				orksmall = 30,
				orkbig = 4,
			},
			team = enemyTeam,
			time = 10,
		},
		{
			name = "intro",
			-- about = "food_healing",
			about = "INFOOOOOOOOOOOO",
			time = 1,
		},
		{
		name = "intro",
		about = "MORE WAVES.",
			time = 3,
		},
		{  -- Night 4
			name = "spawn",
			units = {
				orksmall = 20,
				orkbig = 10,
			},
			team = enemyTeam,
			time = 10,
		},
		{  -- Night 5
			name = "spawn",
			units = {
				orksmall = 50,
				orkbig = 10,
			},
			team = enemyTeam,
			time = 10,
		},
		{  -- Night 6
			name = "spawn",
			units = {
				orksmall = 200,
			},
			team = enemyTeam,
			time = 10,
		},
		{  -- Night 7
			name = "spawn",
			units = {
				orksmall = 50,
				orkbig = 20,
				orkboss = 1,
			},
			team = enemyTeam,
			time = 1,
		},
		{
		name = "outro",
		},
	}
end

local function findPosition(defName)
	local area = spawnAreas[math.random(#spawnAreas)]
	local x = math.random(area.minx, area.maxx)
	local z = math.random(area.minz, area.maxz)
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

function DoIntro(about)
	Spring.SetGameRulesParam("introEvent", about)
end

function DoOutro()
	if Spring.GetGameRulesParam("gameEnd") ~= "loss" then
		Spring.SetGameRulesParam("gameEnd", "victory")
	end
end

function DoStep(step)
	if step.name == "spawn" then
		SpawnUnits(step.units, step.team)
	elseif step.name == "intro" then
		DoIntro(step.about)
	elseif step.name == "outro" then
		DoOutro()
	end
end

function NextStep()
	Spring.Log(LOG_SECTION, LOG.NOTICE, "Next step")
	local step = story[1]
	DoStep(step)
	table.remove(story, 1)
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
