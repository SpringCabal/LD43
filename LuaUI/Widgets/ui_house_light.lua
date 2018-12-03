function widget:GetInfo()
	return {
		name 	= "House lights",
		desc	= "Display lights around houses",
		author	= "gajop",
		date	= "December 2018",
		license	= "GNU GPL, v2 or later",
		layer	= 0,
		enabled = true
	}
end

--if true then return end

local houseDefID = UnitDefNames["house"].id

-- unitID -> lights array
local houses = {}

local minx, maxx, minz, maxz

function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID ~= houseDefID then
		return
	end

	local x, y, z = Spring.GetUnitPosition(unitID)
	local lights = {}
	for _, dx in ipairs({minx, maxx}) do
		for _, dz in ipairs({minz, maxz}) do
			if math.random() > 0.6 then
				local light = {py = y + 50, param = {r = 0.7, g = 0.7, b = 0.4, radius = 500}, colMult = 2}
				light.px = x + dx
				light.pz = z + dz
				table.insert(lights, light)
			end
		end
	end
	houses[unitID] = lights
end

function widget:UnitDestroyed(unitID)
	houses[unitID] = nil
end

local function GetLight(beamLights, beamLightCount, pointLights, pointLightCount)
	for _, lights in pairs(houses) do
		for i, light in ipairs(lights) do
			pointLightCount = pointLightCount + 1
			pointLights[pointLightCount] = light
		end
	end
	return beamLights, beamLightCount, pointLights, pointLightCount
end

local function GetUnitDefSize(unitDefID)
	-- local unitDef = UnitDefs[unitDefID]
	-- local minx, maxx = unitDef.model.minx or -10, unitDef.model.maxx or 10
	-- local minz, maxz = unitDef.model.minz or -10, unitDef.model.maxz or 10
	-- if maxx - minx < 20 then
	-- 	minx, maxx = -10, 10
	-- end
	-- if maxz - minz < 20 then
	-- 	minz, maxz = -10, 10
	-- end
	-- return minx, maxx, minz, maxz

	local sx = 80
	local sz = 80
	return -sx, sx, -sz, sz
end

function widget:Initialize()
	vsx, vsy = Spring.GetViewGeometry()
	minx, maxx, minz, maxz = GetUnitDefSize(houseDefID)

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