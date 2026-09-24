function widget:GetInfo()
	return {
		name    = "RPG Control",
		desc    = "Sends actions to Luarules to control a unit like an RPG character.",
		author  = "Gajop, GoogleFrog",
		date    = "December 2 2018",
		license = "GNU GPL, v2 or later",
		layer   = 1000,
		handler = true,
		enabled = true
	}
end

-------------------------------------------------------------------
-------------------------------------------------------------------
require("keysym.lua")
local UP = KEYSYMS.UP
local DOWN = KEYSYMS.DOWN
local LEFT = KEYSYMS.LEFT
local RIGHT = KEYSYMS.RIGHT
local W = KEYSYMS.W
local S = KEYSYMS.S
local A = KEYSYMS.A
local D = KEYSYMS.D
local SPACE = KEYSYMS.SPACE

local MOVE = "movement"
local ATTACK = "attack"
local ENEMY_TEAM = 1

local controlledDefID = UnitDefNames["bloodmage"].id
local controlledID = nil

local height

local pressSpace = 1
local heldButton

local function getMouseCoordinate(mx,my)
	local traceType, pos = Spring.TraceScreenRay(mx, my, true)
	if not pos then return nil end
	local x, y, z = pos[1], pos[2], pos[3]
-- 	if x<2048 or z<2048 or x>8192 or z>8192 then
-- 		return false
-- 	end
	return x,y,z
end

local function HoldMouse(command, mx, my)
	local traceType, pos = Spring.TraceScreenRay(mx, my, true, false)
	if not pos then
		return false
	end

	if pos and pos[1] then
		Spring.SendLuaRulesMsg(command .. '|' .. pos[1] .. '|' .. pos[3])
	end
end

local function MouseControl()
	if not heldButton then
		return
	end

	local mx, my = Spring.GetMouseState()
	if heldButton == 1 then
		HoldMouse(MOVE, mx, my)
	elseif heldButton == 3 then
		HoldMouse(ATTACK, mx, my)
	end
end

function widget:MousePress(mx, my, button)
	if Spring.GetGameRulesParam("gameMode") == "develop" then
		return false
	end

	if button == 1 or button == 3 then
		heldButton = button
		return true
	end
	return false
end

function widget:MouseMove(mx, my, dx, dy, button)
	if Spring.GetGameRulesParam("gameMode") ~= "develop" and heldButton then
		return true
	end
	return false
end

function widget:MouseRelease(mx, my, button)
	if button == heldButton then
		heldButton = nil
	end

	if Spring.GetGameRulesParam("gameMode") ~= "develop" and
		(button == 1 or button == 3) then
		Spring.SendLuaRulesMsg("stop")
		return true
	end
	return false
end

function widget:SelectionChanged()
	if Spring.GetGameRulesParam("gameMode") ~= "develop" then
		local selected = Spring.GetSelectedUnits()
		if selected and #selected > 0 then
			Spring.SelectUnitArray({})
		end
	end
end

function widget:GameFrame()
	if not controlledID then
		return
	end

	if Spring.GetGameRulesParam("has_eyes") == 1 and pressSpace == 1 then
		pressSpace = 2
	end
	if pressSpace == 3 and Spring.GetGameRulesParam("game_end") == 1 then
		pressSpace = 4
	end
	if pressSpace == 4 and Spring.GetGameRulesParam("game_end") == 0 and Spring.GetGameRulesParam("has_eyes") == 0 then
		pressSpace = 1
	end

	MouseControl()
end

function widget:DrawScreen()
	if pressSpace == 2 then
		gl.PushMatrix()
			gl.Text("Press Space", vsx * 0.4, vsy * 0.4, 20)
		gl.PopMatrix()
	end
end

-- handles weapon switching and abilities
function widget:KeyPress(key, mods, isRepeat)
	if Spring.GetGameRulesParam("gameMode") == "develop" then
		return false
	end

	if not controlledID then
		return false
	end

	if key == S then
		Spring.SendLuaRulesMsg('stop')
	elseif key >= KEYSYMS.N_1 and key <= KEYSYMS.N_5 then
		if isRepeat then
			return true
		end
		local num = key - KEYSYMS.N_1 + 1
		local mx, my = Spring.GetMouseState()
		local x,y,z = getMouseCoordinate(mx,my)
		if not x then
			x, y, z = Spring.GetUnitPosition(controlledID)
		end
		if not x then
			return true
		end
		Spring.SendLuaRulesMsg('spell|' .. tostring(num) .. '|' ..
			tostring(x) .. '|' .. tostring(y) .. '|' .. tostring(z))
	else
		return
	end
	return true
end

function widget:KeyRelease(key)
	if not (Spring.GetKeyState(S)) then
		keyControl = false
	end
end

function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == controlledDefID then
		controlledID = unitID
	end
end

function widget:UnitDestroyed(unitID, unitDefID, unitTeam)
	if controlledID == unitID then
		controlledID = nil
	end
end

function widget:Initialize()
	vsx, vsy = Spring.GetViewGeometry()

	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		widget:UnitCreated(unitID, unitDefID)
	end
end
