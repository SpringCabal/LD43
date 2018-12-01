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

function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == controlledDefID then
		controlledID = unitID
	end
end

function widget:UnitDestroyed(unitID)
	if controlledID == unitID then
		controlledID = nil
	end
end

local function GetLight(beamLights, beamLightCount, pointLights, pointLightCount)
	if controlledID then
		local x, y, z = Spring.GetUnitPosition(controlledID)
		pointLightCount = pointLightCount + 1
		pointLights[pointLightCount] = {px = x, py = y + 50, pz = z, param = {r = 4, g = 1, b = 1, radius = 2000}, colMult = 1}
	end

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