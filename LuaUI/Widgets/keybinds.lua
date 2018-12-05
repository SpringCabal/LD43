function widget:GetInfo()
	return {
		name      = "Keybinds",
		desc      = "",
		author    = "Bluestone",
		date      = "in the future",
		license   = "GPL-v2",
		layer     = -10000,
		enabled   = true,
	}
end

include("keysym.h.lua")

local Chili, screen0
local children = {}
local x,y,h

local purple = "\255\255\10\255"
local white = "\255\255\255\255"


local bindText = { -- keybinds told to player
	--purple .. "Q : " .. white .. "swap pull / push",
	purple .. "1 : " .. white .. "Transfusion",
	purple .. "2 : " .. white .. "Heartburn",
	purple .. "3 : " .. white .. "Adrenaline",
	purple .. "4 : " .. white .. "Migraine",
	purple .. "5 : " .. white .. "Poultice",
	purple .. "Left click: " .. white .. "Move",
	purple .. "Right click : " .. white .. "Attack",
	purple .. "Ctrl+Q : " .. white .. "Quit",
}

local bindDesc = {
	'Gain health at the expense of others.',
	'Causes a burning sensation.',
	'Really gets you (and your allies) moving.',
	'Unbearable headaches for your foes.',
	'Draws orks towards you.',
}

WG.Bindings = bindText
WG.BindingDescription = bindDesc


local function SetBindings()
	local binds = { --real keybinds
		 "Any+pause  pause",
		 --"Alt+b  debug",
		--"--Alt+v  debugcolvol",

		"ctrl+q quitforce",

		"f12 screenshot",
	}
	 for _,binding in pairs(binds) do
		Spring.SendCommands("bind ".. binding)
	end
end

local gameMode
local function UpdateGameMode()
	Spring.SendCommands("unbindall") --muahahahaha
	-- debug console
	Spring.SendCommands('bind f8 toggleErrorConsole')
	if gameMode == "play" then
		Spring.SendCommands("unbindkeyset enter chat") --because because.
		Spring.SendCommands("unbind enter") --because because.
		Spring.SendCommands("unbindaction chat") --because because.
	else
		Spring.SendCommands("bindkeyset enter chat")
		Spring.SendCommands("bind f1 showelevation")
		Spring.SendCommands("bind f2 showpathtraversability")
		Spring.SendCommands("bind Alt+n debug")
		Spring.SendCommands("bind Alt+b debugcolvol")
		Spring.SendCommands("bind a fight")
		Spring.SendCommands("bind p patrol")
		Spring.SendCommands("bind e reclaim")
		Spring.SendCommands("bind any+[ buildfacing inc")
		Spring.SendCommands("bind any+] buildfacing dec")
		Spring.SendCommands("bind w buildunit_house")
		Spring.SendCommands("bind ` drawinmap")
		Spring.SendCommands("bind Ctrl+d selfd")
		Spring.SendCommands("bind Alt+numpad+ speedup")
		Spring.SendCommands("bind Alt+numpad- slowdown")
		Spring.SendCommands("bind any++ speedup")
		Spring.SendCommands("bind any+- slowdown")
	end
	SetBindings()
end

local function cheatAll()
	if gameMode == "play" then
		return
	end
	if not Spring.IsCheatingEnabled() then
		Spring.SendCommands{"cheat"}
	end
	Spring.SendCommands{"spectator"}
	if not Spring.IsGodModeEnabled() then
		Spring.SendCommands{"godmode"}
	end
end

function widget:KeyPress(key, modifier, isRepeat)
	if modifier.ctrl then
		if key == KEYSYMS.G then
			--Spring.SendCommands{"give empiricaldpser 1"} -- Replace with some common test unit.
		end
		if key == KEYSYMS.V then
			cheatAll()
		end
	end
end

function widget:Initialize()
	gameMode = Spring.GetGameRulesParam("gameMode")
	UpdateGameMode()
	if (not WG.Chili) then
		return
	end
	Chili = WG.Chili
	screen0 = Chili.Screen0
	MakeBindingText()
end

function widget:Shutdown()
	for _, ch in pairs(children) do
		ch:Dispose()
	end
end

function MakeBindingText()
	if (not WG.Chili) then
		return
	end
	 for _,child in pairs(children) do
		screen0:RemoveChild(child)
	end
	h = 24
	y = h*(#bindText)
	x = 10
	for _,text in ipairs(bindText) do
		AddLine(text,x,y)
		y = y - h
	end
end

function AddLine(text, x_, y_)
	children[#children+1] = Chili.Label:New{
		x = x_,
		bottom = y_,
		fontsize = 20,
		parent = screen0,
		caption = text,
	}
end

function widget:Update()
	local newGameMode = Spring.GetGameRulesParam("gameMode")
	if gameMode ~= newGameMode then
		gameMode = newGameMode
		UpdateGameMode()
	end
end

WG.ShowBindingText = function(visible)
	if not children then
		return
	end
	for _, ch in pairs(children) do
		if ch.visible ~= visible then
			ch:SetVisibility(visible)
		end
	end
end