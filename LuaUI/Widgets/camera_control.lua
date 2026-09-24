--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function widget:GetInfo()
	return {
		name      = "Camera control",
		desc      = "Controls camera zooming and panning",
		author    = "gajop",
		date      = "WIP",
		license   = "GPLv2",
		version   = "0.1",
		layer     = -1000,
		enabled   = true,  --  loaded by default?
		handler   = true,
		api       = true,
		hidden    = true,
	}
end

local controlledDefID = UnitDefNames["bloodmage"].id
local controlledID = nil
local trackedID = nil
local gameMode

local function StopTracking()
	if trackedID then
		Spring.SendCommands("track off")
		trackedID = nil
	end
end

local function StartTracking()
	if gameMode == "develop" or not controlledID or
		not Spring.ValidUnitID(controlledID) then
		return
	end

	if trackedID ~= controlledID then
		-- Build the tracker group from the selection, as the engine's `track`
		-- action expects. Clear it immediately so it does not leak into the
		-- game UI or alter the hover-only target presentation.
		Spring.SelectUnitArray({controlledID})
		Spring.SendCommands("track off", "track on")
		Spring.SelectUnitArray({})
		trackedID = controlledID
	end

	-- The tracker can be disabled by camera/minimap input. Keep the intended
	-- Lua camera behavior alive by updating the target through the same public
	-- API every frame, just as the game camera is locked to the mage.
	local x, y, z = Spring.GetUnitPosition(controlledID)
	if x then
		Spring.SetCameraTarget(x, y, z, 0)
	end
end

function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == controlledDefID then
		controlledID = unitID
		trackedID = nil
	end
end

function widget:UnitDestroyed(unitID)
	if controlledID == unitID then
		StopTracking()
		controlledID = nil
	end
end

local function SetGameMode(mode)
	if mode == "develop" then
		local s = {
			dist = 2018.541626,
			px = 10.2821,
			py = 436.300781,
			pz = 271.06079,
			rz = 0,
			dx = 0,
			dy = -0.8283768,
			dz = -0.5601712,
			fov = 45,
			ry = 0.00,
			mode = 1,
			rx = 2.54700017,
		}
		Spring.SetCameraState(s, 0)
	else
		local s = {
			dist = 1000,
			px = 5580,
			py = 436.300781,
			pz = 7100,
			rz = 0,
			dx = 0,
			dy = -0.8283768,
			dz = -0.5601712,
			fov = 45,
			ry = 0.00,
			mode = 2,
			rx = 2.5,
		}
		Spring.SetCameraState(s, 0)
	end
end
function widget:Update()
	local newGameMode = Spring.GetGameRulesParam("gameMode")
	if gameMode ~= newGameMode then
		gameMode = newGameMode
		Spring.Echo("Set Game Mode: " .. gameMode)
		SetGameMode(gameMode)
	end
	if gameMode == "develop" then
		StopTracking()
	else
		StartTracking()
	end
end

function widget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		widget:UnitCreated(unitID, unitDefID)
	end
	for k, v in pairs(Spring.GetCameraState()) do
	print(k .. " = " .. tostring(v) .. ",")
	end
	gameMode = Spring.GetGameRulesParam("gameMode")
	SetGameMode(gameMode)
	StartTracking()
end

function widget:Shutdown()
	StopTracking()
end

function widget:MouseWheel(up,value)
	-- uncomment this to disable zoom/panning
	if Spring.GetGameRulesParam("gameMode") ~= "develop" and controlledID ~= nil then
		return true
	end
end
