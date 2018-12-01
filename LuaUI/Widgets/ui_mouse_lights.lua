function widget:GetInfo()
	return {
		name 	= "Mouse lights",
		desc	= "Sends controlled unit actions from LuaUI to LuaRules",
		author	= "gajop",
		date	= "December 2018",
		license	= "GNU GPL, v2 or later",
		layer	= 0,
		enabled = true
	}
end

local function getMouseCoordinate(mx,my)
	local traceType, pos = Spring.TraceScreenRay(mx, my, true)
    if not pos then return false end
	local x, y, z = pos[1], pos[2], pos[3]
-- 	if x<2048 or z<2048 or x>8192 or z>8192 then
-- 		return false
-- 	end
	return x,y,z
end

-- Custom lighting
local function GetMouseLight(beamLights, beamLightCount, pointLights, pointLightCount)
	local mx, my, lmb, mmb, rmb = Spring.GetMouseState()
	local x,y,z = getMouseCoordinate(mx,my)

	if x then
		pointLightCount = pointLightCount + 1
		pointLights[pointLightCount] = {px = x, py = y + 100, pz = z, param = {r = 0.9, g = 0.9, b = 0.9, radius = 5000}, colMult = 1}
	end

	return beamLights, beamLightCount, pointLights, pointLightCount
end

-- local lights = {}
-- local lightDefID = UnitDefNames["light"].id
function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == controlledDefID then
		controlledID = unitID
	end
	if lightDefID == unitDefID then
		lights[unitID] = true
	end
end

function widget:UnitDestroyed(unitID)
	if controlledID == unitID then
		controlledID = nil
	end
	if lightDefID == unitDefID then
		lights[unitID] = nil
	end
end

local function GetWispLight(beamLights, beamLightCount, pointLights, pointLightCount)
	if controlledID and Spring.GetGameRulesParam("spiritMode") == 1 then
		local x, y, z = Spring.GetUnitPosition(controlledID)
		pointLightCount = pointLightCount + 1
		pointLights[pointLightCount] = {px = x, py = y + 50, pz = z, param = {r = 4, g = 4, b = 4, radius = 2000}, colMult = 1}
	end

	return beamLights, beamLightCount, pointLights, pointLightCount
end

-- local function GetLight(beamLights, beamLightCount, pointLights, pointLightCount)
-- 	for lightID, _ in pairs(lights) do
-- 		if Spring.ValidUnitID(lightID) then
-- 			local x, y, z = Spring.GetUnitPosition(lightID)
-- 			pointLightCount = pointLightCount + 1
-- 			pointLights[pointLightCount] = {px = x, py = y + 50, pz = z, param = {r = 0.5, g = 1, b = 1, radius = 1500}, colMult = 1}
-- 		end
-- 	end

-- 	return beamLights, beamLightCount, pointLights, pointLightCount
-- end

function widget:Initialize()
	vsx, vsy = Spring.GetViewGeometry()

	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		widget:UnitCreated(unitID, unitDefID)
	end

	if WG.DeferredLighting_RegisterFunction then
		WG.DeferredLighting_RegisterFunction(GetMouseLight)
		WG.DeferredLighting_RegisterFunction(GetWispLight)
		-- WG.DeferredLighting_RegisterFunction(GetLight)
	end
end

-- function widget:Initialize()
-- -- 	if WG.DeferredLighting_RegisterFunction then
-- -- 		WG.DeferredLighting_RegisterFunction(GetMouseLight)
-- -- 		WG.DeferredLighting_RegisterFunction(GetWispLight)
-- -- 	end
-- end