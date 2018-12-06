function widget:GetInfo()
	return {
		name 	= "Controlled unit lights",
		desc	= "Display lights around the controlled unit",
		author	= "gajop",
		date	= "December 2018",
		license	= "GNU GPL, v2 or later",
		layer	= 0,
		enabled = true
	}
end

local controlledDefID = UnitDefNames["bloodmage"].id
local controlledID = nil
local lightBoost = 1

local STATIC_FLICKER_RATIO = 0.5

local staffX, staffY, staffZ

function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == controlledDefID then
		controlledID = unitID
		staffX, staffY, staffZ = Spring.GetUnitPiecePosition(unitID, Spring.GetUnitPieceMap(unitID)["Hand_Right"])
	end
end

function widget:UnitDestroyed(unitID)
	if controlledID == unitID then
		controlledID = nil
	end
end

local function GetLight(beamLights, beamLightCount, pointLights, pointLightCount)
	if not controlledID then
		return beamLights, beamLightCount, pointLights, pointLightCount
	end

	flicker = math.sin(os.clock() * 3) / 3.14 * STATIC_FLICKER_RATIO
	local x, y, z = Spring.GetUnitViewPosition(controlledID)

	local castingStart = Spring.GetGameRulesParam("start_cast")
	local castingAnimTime = Spring.GetGameRulesParam("castAnimTime")
	if castingStart and castingAnimTime then
		local curr = Spring.GetGameFrame() - castingStart
		if curr <= castingAnimTime*0.4 then
			lightBoost = lightBoost + 0.45
			if lightBoost > 3 then
				lightBoost = 3
			end
		elseif curr <= castingAnimTime*0.55 then
			lightBoost = lightBoost + 0.18
		end
	end
	
	if lightBoost > 1 then
		lightBoost = lightBoost - 0.18
		if lightBoost < 1 then
			lightBoost = 1
		end
	end

	pointLightCount = pointLightCount + 1
	pointLights[pointLightCount] = {
		px = x + staffX,
		py = y + staffY + 15,
		pz = z + staffZ,
		param = {
			r = 4,
			g = 1,
			b = 1,
			radius = 1000
		},
		colMult = 2 * lightBoost * (1 + flicker),
	}

	return beamLights, beamLightCount, pointLights, pointLightCount
end

function widget:Initialize()
	vsx, vsy = Spring.GetViewGeometry()

	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		widget:UnitCreated(unitID, unitDefID)
	end

	if WG.DeferredLighting_RegisterFunction then
		WG.DeferredLighting_RegisterFunction(GetLight)
	end
end

function widget:Shutdown()
	if WG.DeferredLighting_RegisterFunction then
		WG.DeferredLighting_UnregisterFunction(GetLight)
	end
end