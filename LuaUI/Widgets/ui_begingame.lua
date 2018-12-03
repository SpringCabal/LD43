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
You are a respected blood mage in a large village/small town. You have the role of a herbalists, making minor domestically useful magical effects by sacrificing/using specialist animal parts that you buy from hunters/whatever. You are part of the village and local economy.

For some reason the town is attacked by monsters, wave after wave, night after night. You see that the town is not going to survive. To defend the town you decide to open the book of Forbidden Magics that you found during your training at the academy. These magics are much more powerful and surely can aid the defense of the town: throw fireballs, summon lightning, create bogs. However, they are powered by something more advanced than mere animals.

]] .. '\255\255\190\190Sacrifices must be made.\b'


local goalTxt = [[
The game is won by surviving a certain number of nights. The nights would increase in difficulty, there would be a boss at the end, and there would be around 7 nights.
The secondary goal (score) is to save as many of the villagers as you can.
The partial goal is to survive for as many nights as you can.
This goal is designed to make the sacrifices important, since if the game is winnable some of the sacrifices may turn out to have not been necessary. Contrast this with an infinite wave mode in which the aim is to survive for as long as possible. Sacrifices mean less when everyone dies anyway.
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

    Chili.Label:New{
        align = "center",
        x = 0,
        right = 0,
        y = 30,
        parent = mainWindow,
        caption = "BloodMage",
        fontsize = 50,
        textColor = {1,0,0,1},
    }

    local controlsTxt = ""
    local startY = 100
    Chili.Label:New {
        x = 10,
        y = startY,
        fontsize = 20,
        parent = mainWindow,
        caption = "Controls:",
    }
    for i, text in pairs(WG.Bindings) do
        Chili.Label:New{
            x = 10,
            y = startY + i * 30,
            fontsize = 20,
            parent = mainWindow,
            caption = text,
        }
        local desc = WG.BindingDescription[i]
        if desc then
            Chili.Label:New {
                x = 200,
                y = startY + i * 30,
                fontsize = 20,
                parent = mainWindow,
                caption = desc,
            }
        end
    end

    Chili.ScrollPanel:New {
        x = 500, right = 0,
        y = 120, height = 150,
        parent = mainWindow,
        children = {
            Chili.TextBox:New {
                x = 0, right = 0,
                text = storyTxt,
                fontsize = 16,
            },
        }
    }

    Chili.ScrollPanel:New {
        x = 500, right = 0,
        y = 300, height = 150,
        parent = mainWindow,
        children = {
            Chili.TextBox:New {
                x = 0, right = 0,
                text = goalTxt,
                fontsize = 16,
            },
        }
    }

    Chili.Button:New {
        bottom  = 30,
        width   = 200,
        x       = 150,
        height  = 55,
        caption = "Play",
        fontsize = 20,
        OnClick = {
            ActionOK
        },
        parent = mainWindow,
    }
    Chili.Button:New {
        bottom  = 30,
        width   = 200,
        right   = 150,
        height  = 55,
        caption = "Exit",
        fontsize = 20,
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

function widget:Shutdown()
    if mainWindow then
        mainWindow:Dispose()
    end
end