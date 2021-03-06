if addon.InGetInfo then
	return {
		name      = "Remove Engine Menu";
		desc      = "Removes the engines build/command menu";
		author    = "Bluestone";
		date      = "2008-2013";
		license   = "GNU GPL, v2 or later";

		layer     = math.huge;
		hidden    = true; -- don't show in the widget selector
		api       = true; -- load before all others?
		enabled   = true; -- loaded by default?
	}
end

local USE_CTRL_PANEL = false

local function DummyHandler(xIcons, yIcons, cmdCount, commands)
	handler.commands   = commands
	handler.commands.n = cmdCount
	handler:CommandsChanged()
	return "", xIcons, yIcons, {}, {}, {}, {}, {}, {}, {}, {}
end

function addon.Initialize()
	if USE_CTRL_PANEL then
		Spring.LoadCtrlPanelConfig([[xIcons         3
yIcons         10
prevPageSlot   j0
deadIconSlot   j1
nextPageSlot   j2
xIconSize      0.05
yIconSize      0.05

xPos           0.000
yPos           0.147
xSelectionPos  0.018
ySelectionPos  0.115]])
		Spring.LoadCmdColorsConfig([[alwaysDrawQueue 1]])
		Spring.ForceLayoutUpdate()

	else
		RegisterGlobal("LayoutButtons", DummyHandler)
	end
end

