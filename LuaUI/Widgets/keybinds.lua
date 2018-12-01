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

local bindText, mouseText
local Chili, screen0
local children = {}
local x,y,h

local purple = "\255\255\10\255"
local white = "\255\255\255\255"

local function SetBindings()
    local binds = { --real keybinds

        "Any+pause  pause",

        --"Alt+b  debug",
        --"--Alt+v  debugcolvol",

		"ctrl+q quitforce",

		"f12 screenshot",

		"Any+up moveforward ",
		"Any+left moveleft ",
		"Any+right moveright",
		"Any+down moveback",

		"Any+w moveforward ",
		"Any+a moveleft ",
		"Any+d moveright",
		"Any+s moveback",

		"Any+pageup moveup",
		"Any+pagedown movedown",
    }

    for _,binding in pairs(binds) do
        Spring.SendCommands("bind ".. binding)
    end
end

local gameMode
local function UpdateGameMode()
    Spring.SendCommands("unbindall") --muahahahaha
    if gameMode == "play" then
        Spring.SendCommands("unbindkeyset enter chat") --because because.
    else
        Spring.SendCommands("bindkeyset enter chat")
        Spring.SendCommands("bind f1 showelevation")
        Spring.SendCommands("bind f2 showpathtraversability")
    end
    SetBindings()
end

function widget:Initialize()
    gameMode = Spring.GetGameRulesParam("gameMode")
    UpdateGameMode()

    bindText = { -- keybinds told to player
        --purple .. "Q : " .. white .. "swap pull / push",
        purple .. "1 : " .. white .. "Haste",
        purple .. "2 : " .. white .. "Fireball",
        purple .. "3 : " .. white .. "Web",
        purple .. "Ctrl+Q : " .. white .. "Quit",
    }

    mouseText = {
--         purple .. "Left click: " .. white .. "Shoot",
--         purple .. "Right click : " .. white .. "Place Mines",
--         purple .. "Scroll Wheel : " .. white .. "Zoom",
    }


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


    h = 20
    y = h*(#bindText + #mouseText)
    x = 10

    for _,text in ipairs(mouseText) do
        AddLine(text,x,y)
        y = y - h
    end
    for _,text in ipairs(bindText) do
        AddLine(text,x,y)
        y = y - h
    end
end

function  AddLine(text, x_, y_)
    children[#children+1] = Chili.Label:New{
        x = x_,
        bottom = y_,
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
