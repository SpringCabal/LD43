--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function widget:GetInfo()
  return {
	name      = "Chili EndGame Window",
	desc      = "Derived from v0.005 Chili EndGame Window by CarRepairer",
	author    = "Anarchid",
	date      = "April 2015",
	license   = "GNU GPL, v2 or later",
	layer     = 0,
	enabled   = true,
  }
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
local spSendCommands			= Spring.SendCommands

local echo = Spring.Echo

local Chili
local Image
local Button
local Checkbox
local Window
local Panel
local ScrollPanel
local StackPanel
local Label
local screen0
local color2incolor
local incolor2color

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local window_endgame
local frame_delay = 0
local sentGameStart = false
local playerName = ""
local nameBox, restartButton, submitButton, lblUpload
local gameOverTime

local controlledDefID = UnitDefNames["bloodmage"].id
local controlledID = nil

local function ShowEndGameWindow()
	screen0:AddChild(window_endgame)
end

local function SetupControls(isVictory)
	local winSizeX, winSizeY = Spring.GetWindowGeometry()
	local width, height = 400, 400

	window_endgame = Window:New {
		name = "GameOver",
		--caption = "Game Over",
		x = (winSizeX - width)/2,
		y = winSizeY/2 - height*0.65,
		width  = width,
		height = height,
		padding = {8, 8, 8, 8};
		--autosize   = true;
		--parent = screen0,
		draggable = false,
		resizable = false,
	}
	local score = Spring.GetGameRulesParam("score") or 0
	local survialTime = Spring.GetGameRulesParam("survivalTime") or 0
	local rabbitKills = Spring.GetGameRulesParam("rabbits_killed") or 0

	caption = Chili.Label:New{
		x = 10,
		y = (isVictory and 5) or 25,
		right = 10,
		parent = window_endgame,
		align = "center",
		caption = (isVictory and "The Town is\n    Saved!") or "Defeat!",
		fontsize = 50,
		textColor = (isVictory and {0,1,0,1}) or {1,0,0,1},
	}
	
	if isVictory then
		Chili.Label:New{
			x = 20,
			y = 150,
			width = 100,
			parent = window_endgame,
			caption = (Spring.GetGameRulesParam("villagersKilled") or 0) .. " townsfolk lost their lives.",
			fontsize = 20,
			textColor = {1,1,1,1},
		}
		Chili.Label:New{
			x = 20,
			y = 175,
			width = 100,
			parent = window_endgame,
			caption = (Spring.GetGameRulesParam("alliesDrained") or 0) .. " of the fallen were found mysteriously\ndrained of blood.",
			fontsize = 20,
			textColor = {1,1,1,1},
		}
	else
		Chili.Label:New{
			x = 20,
			y = 105,
			width = 100,
			parent = window_endgame,
			caption = (Spring.GetGameRulesParam("orksKilled") or 0) .. " orks were slain before you\nsuccumbed.",
			fontsize = 20,
			textColor = {1,1,1,1},
		}
		Chili.Label:New{
			x = 20,
			y = 180,
			width = 100,
			parent = window_endgame,
			caption = "There is no hope without your aid.",
			fontsize = 20,
			textColor = {1,1,1,1},
		}
	end
	
	Chili.Label:New{
		x = 20,
		y = 260,
		width = 100,
		parent = window_endgame,
		caption = "Thanks for playing!",
		-- caption = "Score: " .. score,
		fontsize = 20,
		textColor = {1,1,1,1},
	}
	restartButton = Button:New{
		bottom  = 5,
		width   = 175,
		x       = 10,
		height  = 80,
		caption = "Restart",
		fontsize = 24,
		OnClick = {
			function()
				nameBox = nil
				restartButton = nil
				submitButton = nil
				Spring.SendCommands("cheat 1", "luarules reload", "cheat 0")
				window_endgame:Dispose()
				window_endgame = nil
				frame_delay = Spring.GetGameFrame()
			end
		},
		parent = window_endgame,
	}
	Button:New{
		bottom  = 5,
		width   = 175,
		right   = 10,
		height  = 80,
		caption = "Exit",
		fontsize = 24,
		OnClick = {
			function()
				nameBox = nil
				restartButton = nil
				Spring.SendCommands("quit", "quitforce")
			end
		},
		parent = window_endgame,
	}

	screen0:AddChild(window_endgame)
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
--callins

include('keysym.h.lua')
local RETURN = KEYSYMS.RETURN
function widget:KeyPress(key, mods, isRepeat)
	if key == RETURN and restartButton and restartButton.OnClick and restartButton.OnClick[1] and
			submitButton and submitButton.OnClick and submitButton.OnClick[1] then
		submitButton.OnClick[1]()
		restartButton.OnClick[1]()
		return true
	end
end

function widget:Initialize()
	if (not WG.Chili) then
		widgetHandler:RemoveWidget()
		return
	end

	Chili = WG.Chili
	Image = Chili.Image
	Button = Chili.Button
	Checkbox = Chili.Checkbox
	Window = Chili.Window
	Panel = Chili.Panel
	ScrollPanel = Chili.ScrollPanel
	StackPanel = Chili.StackPanel
	Label = Chili.Label
	screen0 = Chili.Screen0
	color2incolor = Chili.color2incolor
	incolor2color = Chili.incolor2color
	
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		widget:UnitCreated(unitID, unitDefID)
	end
end

function widget:Shutdown()
	if window_endgame then
		window_endgame:Dispose()
	end
end

function widget:GameFrame()
	if Spring.GetGameRulesParam("gameEnd") == "victory" then
		self:GameOver({Spring.GetMyAllyTeamID()})
	end
end

function widget:GameOver(winningAllyTeams)
	if window_endgame or Spring.GetGameFrame() - frame_delay < 300 then
		return
	end
	if WG.analytics and WG.analytics.SendEvent then
		gameOverTime = os.clock()
		local score = Spring.GetGameRulesParam("score") or 0
		local survivalTime = Spring.GetGameRulesParam("survivalTime") or 0
		local rabbitKills = Spring.GetGameRulesParam("rabbits_killed") or 0
		local shotsFired = Spring.GetGameRulesParam("shots_fired") or 0
		local minesPlaced = Spring.GetGameRulesParam("mines_placed") or 0

		WG.analytics:SendEvent("score", score)
		WG.analytics:SendEvent("time", survivalTime)
		WG.analytics:SendEvent("kills", rabbitKills)
		WG.analytics:SendEvent("shots", shotsFired)
		WG.analytics:SendEvent("mines", minesPlaced)
		WG.analytics:SendEvent("game_end")
	end
	local myAllyTeamID = Spring.GetMyAllyTeamID()
	for _, winningAllyTeamID in pairs(winningAllyTeams) do
		if myAllyTeamID == winningAllyTeamID then
			Spring.SendCommands("endgraph 0")
			SetupControls(true)
			return
		end
	end
	Spring.SendCommands("endgraph 0")
	SetupControls(false)
end

function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == controlledDefID then
		controlledID = unitID
	end
end

function widget:UnitDestroyed(unitID, unitDefID, unitTeam)
	if controlledID == unitID then
		controlledID = nil
		if Spring.GetGameRulesParam("gameMode") ~= "develop" then
			widget:GameOver({})
		end
	end
end