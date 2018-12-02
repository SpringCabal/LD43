function widget:GetInfo()
	return {
		name      = "Deferred rendering test",
		version   = 1,
		desc      = "Spawns a few random lights around the center of the map.",
		author    = "gajop",
		date      = "2018 Sept.",
		license   = "GPL V2",
		layer     = 0,
		enabled   = true
	}
end

local LOG_SECTION = "Deferred rendering test"

local START_LIGHTS = 100


local lights = {}

local function SetupLights()
	for i = 1, START_LIGHTS do
		local x = Game.mapSizeX / 2 + math.random(-1000, 1000)
		local z = Game.mapSizeZ / 2 + math.random(-1000, 1000)
		local y = Spring.GetGroundHeight(x, z) + 200
		table.insert(lights, {
		px = x,
		py = y,
		pz = z,
		 vx = 0,
		vy = 0,
		vz = 0,
		colMult = 3,
		param = {
		radius = 1000,
		r = math.random(),
		g = math.random(),
		b = math.random(),
		}
		})
	end
end

function RenderLights(beamLights, beamLightCount, pointLights, pointLightCount)
	local centerx = Game.mapSizeX / 2
	local centerz = Game.mapSizeZ / 2
	local centery = Spring.GetGroundHeight(centerx, centerz) + 100
	for _, light in ipairs(lights) do
		-- random movements
		light.vx = light.vx + (math.random() - 0.5)
		light.vy = light.vy + (math.random() - 0.5)
		light.vz = light.vz + (math.random() - 0.5)
		 -- leashed to center
		light.vx = light.vx + (centerx - light.px) / 10000
		light.vy = light.vy + (centery - light.py) / 10000
		light.vz = light.vz + (centerz - light.pz) / 10000
		light.px = light.px + light.vx
		light.py = light.py + light.vy
		light.pz = light.pz + light.vz
		pointLightCount = pointLightCount + 1
		pointLights[pointLightCount] = light
	end
	return beamLights, beamLightCount, pointLights, pointLightCount
end

local disabled = true
function widget:Initialize()
	if disabled then
		widgetHandler:RemoveWidget(self)
		return
	end
	 if not WG.DeferredLighting_RegisterFunction then
		Spring.Log(LOG_SECTION, LOG.ERROR, "Missing WG.DeferredLighting_RegisterFunction function. Deferred lights aren't enabled.")
		widgetHandler:RemoveWidget(self)
		return
	end
	Spring.Log(LOG_SECTION, LOG.NOTICE,"Setting up test lights...")
	WG.DeferredLighting_RegisterFunction(RenderLights)
	SetupLights()
end

function widget:Shutdown()
end