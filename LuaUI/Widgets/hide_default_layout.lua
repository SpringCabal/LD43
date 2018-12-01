function widget:GetInfo()
  return {
    name      = "Hides the default interface",
    desc      = "",
    author    = "gajop",
    date      = "in the future",
    license   = "GPL-v2",
    layer     = 1001,
    enabled   = true,
  }
end

local gameMode
local function UpdateGameMode()
    if gameMode == "play" then
        --Spring.SendCommands("Console 0") -- Handled by console widget
    else
        --Spring.SendCommands("Console 1") -- Handled by console widget
    end
end

function widget:Initialize()
    gameMode = Spring.GetGameRulesParam("gameMode")
    UpdateGameMode()

    -- remove Springs default UI stuff
    Spring.SendCommands("ResBar 0", "ToolTip 0", "Clock 0", "Info 0")
    Spring.SendCommands("showhealthbars 0")
	ShareDialog                     
    gl.SlaveMiniMap(true)
    gl.ConfigMiniMap(-1,-1,-1,-1)
end

-- sets status to ready & hide pre-game UI
function widget:GameSetup()
    return true, true
end

function widget:Update()
    local newGameMode = Spring.GetGameRulesParam("gameMode")
    if gameMode ~= newGameMode then
        gameMode = newGameMode
        UpdateGameMode()
    end
end
