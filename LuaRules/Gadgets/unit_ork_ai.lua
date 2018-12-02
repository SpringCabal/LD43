if not gadgetHandler:IsSyncedCode() then
	return
end

function gadget:GetInfo()
   return {
      name      = "ork_ai",
      desc      = "",
      author    = "gajop",
      date      = "LD43",
      license   = "GPL-v2",
      layer     = 0,
      enabled   = true
   }
end

local AI_TESTING_MODE = false

local orkSmallDefID = UnitDefNames["orksmall"].id


local controlledDefID = UnitDefNames["bloodmage"].id
local controlledID = nil

local currentWave = 0
local startSpawnFrame
local spawnPoints = nil

-- story stuff
local treesTakeNoDamage = false
local storyMushrooms = {}
local storyWaveSpawnTime

local firstSpawnFrame = nil

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- Event Handling
--------------------------------------------------------------------------------

function gadget:Initialize()
	Spring.SetGameRulesParam("story", 1)
	Spring.SetGameRulesParam("skip_tutorial", 0)
	Spring.SetGameRulesParam("shroomEvent", 0)
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		self:UnitCreated(unitID, unitDefID)
	end
end

local _loadFrame
function gadget:GameFrame(frame)
   if Spring.GetGameRulesParam("gameDev") == "develop" then
      return
   end

	if frame%15==0 then
        CheckForIdleMushrooms()
    end

	local story = Spring.GetGameRulesParam("story")
	if story ~= 0 then
		if CheckTreeHP() then
			return
		end
		if storyWaveSpawnTime ~= nil and frame - storyWaveSpawnTime > 30 then
			if CheckMushrooms() then
				return
			end
		end
		if story ~= 5 then
			Spring.SetGameRulesParam("mana", 0)
		end
		if CheckUpgradedTree() then
			return
		end

		return
	elseif startSpawnFrame == nil then
		startSpawnFrame = 100 + Spring.GetGameFrame()
	end
	local gameFrame = frame - startSpawnFrame
	if gameFrame < 0 then
		return
	end
	SpawnNextWave()
end


--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- Mushroom AI
--------------------------------------------------------------------------------

local aiUnits = {}

function CheckForSpire(unitID, unitDefID)
	if unitDefID == controlledDefID then
		controlledID = unitID
	end
end

function gadget:UnitCreated(unitID, unitDefID)
	CheckForSpire(unitID, unitDefID)

   if UnitDefs[unitDefID].customParams.ork and not (Spring.GetUnitRulesParam(unitID, "aiDisabled")==1) then
      aiUnits[unitID] = true
   end
end

function gadget:UnitDestroyed(unitID, unitDefID)
	local stage = Spring.GetGameRulesParam("story")
	if stage == 2 then
		storyMushrooms[unitID] = nil
		local c = 0
		for _, _ in pairs(storyMushrooms) do 
			c = c + 1
		end
		if c == 0 then
			StoryStage(3)
		end
	end

	if unitDefID == kingshroomDefID then
		Spring.SetGameRulesParam("shroomEvent", 2)
	end
	aiUnits[unitID] = nil
end

function SelectEnemy(uID, allUnits)
    local nID = Spring.GetUnitNearestEnemy(uID, 10240, true)
    local x,y,z = Spring.GetUnitPosition(uID)
    if math.random()<0.5 and nID ~= controlledID then
        return nID
    else
        -- sample a random enemy with probability proportional to 1 / square distance from self
        local tID = Spring.GetUnitTeam(uID)
        local weights = {}
        local totalWeight = 0
        for i=1,#allUnits do
            local eID = allUnits[i]
            local eTeamID = Spring.GetUnitTeam(eID)
            local eDID = Spring.GetUnitDefID(eID)
            if (UnitDefs[eDID].customParams.tree or controlledDefID == eDID) and not Spring.AreTeamsAllied(tID, eTeamID) then
                local ex,ey,ez = Spring.GetUnitPosition(eID)
                local sqrDist = (x-ex)*(x-ex) + (y-ey)*(y-ey) + (z-ez)*(z-ez) 
                weights[i] = (sqrDist>10*10) and 1/(sqrDist) or 0
                totalWeight = totalWeight + weights[i]
            else
                weights[i] = 0
            end
        end
        if totalWeight<=0 then
            return nil
        end
        local p = math.random()*totalWeight
        local q = 0
        for i=1,#weights do
            q = q + weights[i]
            if q>p then
                return allUnits[i]
            end
        end
    end
    return nil
end

local SMALL_MUSHROOM_ORDER_CHANGE = 30
function CheckForIdleMushrooms()
	local frame = Spring.GetGameFrame()
    local allUnits = Spring.GetAllUnits()
    for uID, _ in pairs(aiUnits) do
		local unitDefID = Spring.GetUnitDefID(uID)
		if unitDefID == orkSmallDefID then
			local lastOrderFrame = Spring.GetUnitRulesParam(uID, "lastOrderFrame") or 0
			if frame - lastOrderFrame >= SMALL_MUSHROOM_ORDER_CHANGE then
				lastOrderFrame = frame
				Spring.SetUnitRulesParam(uID, "lastOrderFrame", lastOrderFrame)

				local eID = SelectEnemy(uID, allUnits)
				if eID then
					local x,y,z = Spring.GetUnitPosition(eID)
					local d = 500
					local dx = x + math.random()*d - d/2
					local dz = z + math.random()*d - d/2
					local dy = Spring.GetGroundHeight(dx, dz)
					Spring.GiveOrderToUnit(uID, CMD.MOVE, {dx, dy, dz}, {})
				end
			end
		elseif Spring.GetUnitCommands(uID,2) ~= nil and #Spring.GetUnitCommands(uID,2)==0 then
            local eID = SelectEnemy(uID, allUnits)
            if eID then
                local x,y,z = Spring.GetUnitPosition(eID)
                local cx,cy,cz = Spring.GetUnitPosition(uID)
                StandUpAndFightLikeAMan(uID,x,y,z)
            else
                GoForAShortWalk(uID)
            end
        end
    end
end

function GoForAShortWalk(uID)
    local x,y,z = Spring.GetUnitPosition(uID)
    local theta = math.random(1,360) / 360 * (2*math.pi)
    local d = 256
    local dx, dz = d*math.sin(theta), d*math.cos(theta)
    local nx, ny, nz = x+dx, Spring.GetGroundHeight(x+dx,z+dz), z+dz
    Spring.GiveOrderToUnit(uID, CMD.FIGHT, {nx,ny,nz}, {})    
end

function StandUpAndFightLikeAMan(uID,x,y,z)
    Spring.GiveOrderToUnit(uID, CMD.FIGHT, {x,y,z}, {})
end


--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- Trees Special Modes
--------------------------------------------------------------------------------

function gadget:UnitPreDamaged(unitID, unitDefID, unitTeam, damage, paralyzer, weaponDefID, projectileID, attackerID, attackerDefID, attackerTeam)
    if not UnitDefs[unitDefID].customParams.tree then
        return damage,0
    end

    if treesTakeNoDamage then
        return 0,0
    end

    return damage, 0
end

function DamageTrees(damageTreeP)
    -- set all trees to have health approximately p of their full health (plus a bit of randomness)
    local units = Spring.GetAllUnits()
    for _,uID in ipairs(units) do
        local unitDefID = Spring.GetUnitDefID(uID)
        if UnitDefs[unitDefID].customParams.tree then
            local h,mh = Spring.GetUnitHealth(uID)
            local deviation = 0.2
            local newH = math.max(0.05*mh, math.min(0.95*mh, mh*damageTreeP + deviation*mh*2*(math.random()-1) ) )
            Spring.SetUnitHealth(uID, newH)
        end
    end
end

function CheckTreeHP()
	local stage = Spring.GetGameRulesParam("story")
	if stage ~= 3 then
		return
	end

	local units = Spring.GetAllUnits()
	local fullHP = true
    for _, uID in ipairs(units) do
        local unitDefID = Spring.GetUnitDefID(uID)
        if UnitDefs[unitDefID].customParams.tree then
            local h,mh = Spring.GetUnitHealth(uID)
			if mh * 0.8 > h then
				fullHP = false
			end
		end
	end
	if fullHP then
		StoryStage(4)
		return true
	end
end

function CheckMushrooms()
	local stage = Spring.GetGameRulesParam("story")
	if stage ~= 4 then
		return
	end
	local noshroomsC = 0
	local units = Spring.GetAllUnits()
	for _, uID in ipairs(units) do
        local unitDefID = Spring.GetUnitDefID(uID)
        if UnitDefs[unitDefID].customParams.mushroom then
            noshroomsC = noshroomsC + 1
		end
	end
	-- there always seems to be a few left...
	if noshroomsC <= 2 then
		StoryStage(5)
		return true
	end
end

function CheckUpgradedTree()
	local stage = Spring.GetGameRulesParam("story")
	if stage ~= 5 then
		return
	end
	local units = Spring.GetAllUnits()
	for _, uID in ipairs(units) do
        local unitDefID = Spring.GetUnitDefID(uID)
        if unitDefID == treeLevel2DefID then
			StoryStage(6)
            return true
		end
	end
end
