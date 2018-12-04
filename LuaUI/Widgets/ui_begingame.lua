--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function widget:GetInfo()
return {
    name      = "Chili BeginGame Window",
    desc      = "Derived from Chili EndGame Window",
    author    = "gajop",
    date      = "LD43",
    license   = "GNU GPL, v2 or later",
    layer     = 0,
    enabled   = true,
}
end

local Chili
local screen0

local mainWindow
local playerName = ""


local storyTxt = [[
Usually the militia is more than a match for the occasional orkish raid. As the towns blood mage your contibuted is limited to patching up any wounded (at a heavy discount, of course). This time is different. Everyone who can weild a pitchfork is being urged to fight. A gigantic army headed by a fearsome (and massive) leader is approaching. There is no way the militia is up to the task, and, alas, your spells and effects are too weak to tip the balance.

Unless... you were to crack open your book of Forbidden Magic. Powered by more than pricked fingers and vials of sheep blood, these spells may just be the tool required to save us. Well, most of us. The townsfolk will understand as, if any of us are to see the dawn...


]] .. '\255\255\190\190Sacrifices must be made.\b'


local goalTxt = [[

 + Protect the townsfolk.

 + Kill the leader of the orks.

 + Survive the night.

 + Try to live with yourself afterwards.
]]

local function ActionOK()
    nameBox = nil
    restartButton = nil
    submitButton = nil
    mainWindow:Dispose()
    mainWindow = nil

    WG.ShowBindingText(true)

    Spring.SendCommands("pause 0")
end

local function ActionCancel()
    Spring.SendCommands("quit", "quitforce")
end

local function SetupControls()
    local winSizeX, winSizeY = Spring.GetWindowGeometry()
    local width, height = 400, 400

    mainWindow = Chili.Window:New {
        name = "GameBegin",

        --caption = "Game Over",
        x = "5%",
        y = "5%",
        width  = "90%",
        height = "90%",
        padding = {8, 8, 8, 8};
        --autosize   = true;
        --parent = screen0,
        draggable = false,
        resizable = false,
    }
	
	Chili.Panel:New {
        --caption = "Game Over",
        x = 0,
        y = 0,
        width  = 0,
        height = 0,
        padding = {8, 8, 8, 8};
        --autosize   = true;
        --parent = screen0,
        draggable = false,
        resizable = false,
		parent = mainWindow,
    }
	Chili.Panel:New {
        --caption = "Game Over",
        x = 0,
        y = 0,
        width  = 0,
        height = 0,
        padding = {8, 8, 8, 8};
        --autosize   = true;
        --parent = screen0,
        draggable = false,
        resizable = false,
		parent = mainWindow,
    }

    Chili.Label:New{
        align = "center",
        x = 0,
        right = 0,
        y = 30,
        parent = mainWindow,
        caption = "Defend Your Town!",
        fontsize = 50,
        textColor = {1,0,0,1},
    }

    local controlsTxt = ""
    local startY = 250
    Chili.Label:New {
        x = 120,
        y = startY + 10,
        fontsize = 32,
        parent = mainWindow,
        caption = "Controls:",
    }
    for i, text in pairs(WG.Bindings) do
        Chili.Label:New{
            x = 120,
            y = startY + i * 50,
            fontsize = 20,
            parent = mainWindow,
            caption = text,
        }
        local desc = WG.BindingDescription[i]
        if desc then
            Chili.Label:New {
                x = 180,
                y = startY + 28 + i * 50,
                fontsize = 16,
                parent = mainWindow,
                caption = desc,
            }
        end
    end

    Chili.ScrollPanel:New {
        x = 500, right = 180,
        y = 120, height = 320,
        parent = mainWindow,
		backgroundColor = {0,0,0,0},
		borderColor = {0,0,0,0},
        children = {
            Chili.TextBox:New {
                x = 0, right = 0,
                text = storyTxt,
                fontsize = 20,
            },
        }
    }

    Chili.ScrollPanel:New {
        x = 500, right = 180,
        y = 420, height = 200,
        parent = mainWindow,
		backgroundColor = {0,0,0,0},
		borderColor = {0,0,0,0},
        children = {
            Chili.TextBox:New {
                x = 0, right = 0,
                text = goalTxt,
                fontsize = 20,
            },
        }
    }

    Chili.Button:New {
        bottom  = 30,
        width   = 350,
        x       = 520,
        height  = 120,
        caption = "Play",
        fontsize = 52,
        OnClick = {
            ActionOK
        },
        parent = mainWindow,
    }
    Chili.Button:New {
        bottom  = 30,
        width   = 350,
        right   = 50,
        height  = 120,
        caption = "Exit",
        fontsize = 52,
        OnClick = {
            ActionCancel
        },
        parent = mainWindow,
    }

    screen0:AddChild(mainWindow)

    WG.ShowBindingText(false)
    Spring.SendCommands("pause 1")
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
--callins

include('keysym.h.lua')
local RETURN = KEYSYMS.RETURN
function widget:KeyPress(key, mods, isRepeat)
    if key == RETURN and mainWindow and mainWindow.visible then
        ActionOK()
        return true
    end
end

function widget:Initialize()
    if not WG.Chili then
        widgetHandler:RemoveWidget()
        return
    end

    Chili = WG.Chili
    screen0 = Chili.Screen0

    SetupControls()
end

local init = false
function widget:GameFrame()
	if init then
		return
	end
	if mainWindow and mainWindow.parent then
		Spring.SendCommands("pause 1")
	end
	init = true
end

function widget:Shutdown()
    if mainWindow then
        mainWindow:Dispose()
    end
end