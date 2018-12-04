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

local gameMode
function SetGameMode(gameMode)
	if Spring.GetGameRulesParam("gameMode") == "develop" then
		s = {
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
		s = {
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
frist = true
lastTrackingUpdate = os.clock()
function widget:Update()
	if Spring.GetGameRulesParam("gameMode") ~= "develop" and controlledID ~= nil then
		Spring.SelectUnitArray({controlledID})
		Spring.SendCommands({"trackoff", "track"})
		if(frist) then
		Spring.SendCommands({"trackmode 1"});
		frist = false
		end
		Spring.SelectUnitArray({})
	end
	local newGameMode = Spring.GetGameRulesParam("gameMode")
	if gameMode ~= newGameMode then
		gameMode = newGameMode
		Spring.Echo("Set Game Mode: " .. gameMode)
		SetGameMode(gameMode)
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
end

function widget:Shutdown()
end

function widget:MouseWheel(up,value)
	-- uncomment this to disable zoom/panning
	if Spring.GetGameRulesParam("gameMode") ~= "develop" and controlledID ~= nil then
		return true
	end
end
