function widget:GetInfo()
	return {
		name 	= "Mouse lights",
		desc	= "Display light around the mouse",
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

function widget:Initialize()
	vsx, vsy = Spring.GetViewGeometry()

	if WG.DeferredLighting_RegisterFunction then
		WG.DeferredLighting_RegisterFunction(GetMouseLight)
	end
end

function widget:Shutdown()
	if WG.DeferredLighting_RegisterFunction then
		WG.DeferredLighting_UnregisterFunction(GetMouseLight)
	end
end