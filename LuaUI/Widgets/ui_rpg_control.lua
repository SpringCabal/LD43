function widget:GetInfo()
	return {
		name    = "RPG Control",
		desc    = "Sends actions to Luarules to control a unit like an RPG character.",
		author  = "Gajop, GoogleFrog",
		date    = "December 2 2018",
		license = "GNU GPL, v2 or later",
		layer   = 0,
		enabled = true
	}
end

-------------------------------------------------------------------
-------------------------------------------------------------------
local mouseControl1 = false
local mouseControl3 = false
include('keysym.h.lua')
local UP = KEYSYMS.UP
local DOWN = KEYSYMS.DOWN
local LEFT = KEYSYMS.LEFT
local RIGHT = KEYSYMS.RIGHT
local W = KEYSYMS.W
local S = KEYSYMS.S
local A = KEYSYMS.A
local D = KEYSYMS.D
local SPACE = KEYSYMS.SPACE

local controlledDefID = UnitDefNames["bloodmage"].id
local controlledID = nil

local height

local pressSpace = 1

local function getMouseCoordinate(mx,my)
	local traceType, pos = Spring.TraceScreenRay(mx, my, true)
	if not pos then return false end
	local x, y, z = pos[1], pos[2], pos[3]
-- 	if x<2048 or z<2048 or x>8192 or z>8192 then
-- 		return false
-- 	end
	return x,y,z
end

local function MouseControl()
	local mx, my, lmb, mmb, rmb = Spring.GetMouseState()
Spring.Echo(mx, my)
	if lmb and mouseControl1 then
		local x,y,z = getMouseCoordinate(mx,my)
		if (x) then
			Spring.SendLuaRulesMsg('movement|' .. x .. '|' .. z)
			return true
		end
	elseif rmb and mouseControl3 then
		local x,y,z = getMouseCoordinate(mx,my)
		if (x) then
			Spring.SendLuaRulesMsg('movement|' .. x .. '|' .. z)
			return true
		end
	end
end

function widget:MousePress(mx, my, button)
	if Spring.GetGameRulesParam("gameMode") == "develop" then
		return false
	end

	if Spring.IsAboveMiniMap(mx, my) then
		return false
	end

	local alt, ctrl, meta, shift = Spring.GetModKeyState()
	if button == 1 then
		local x,y,z = getMouseCoordinate(mx,my)
		Spring.SendLuaRulesMsg('attack|' .. x .. '|' .. y .. '|' .. z)
		return true
	elseif button == 3 then
		local x,y,z = getMouseCoordinate(mx,my)
		height = Spring.GetGroundHeight(x, z)
		if (x) then
			Spring.SendLuaRulesMsg('dec_heightmap|' .. x .. '|' .. y .. '|' .. z .. "|" .. height)
			mouseControl3 = true
			return true
		else
			return false
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

	if mouseControl1 or mouseControl3 then
		MouseControl()
	end
end

function widget:DrawScreen()
	if pressSpace == 2 then
		gl.PushMatrix()
			gl.Text("Press Space", vsx * 0.4, vsy * 0.4, 20)
		gl.PopMatrix()
	end
end

function widget:MouseRelease(mx, my, button)
	if button == 1 then
		mouseControl1 = false
	elseif button == 3 then
		mouseControl3 = false
	end
end

-- handles weapon switching and abilities
function widget:KeyPress(key, mods, isRepeat)
	if Spring.GetGameRulesParam("gameMode") == "develop" then
		return false
	end

	if not controlledID then
		return
	end

	if key == LEFT or key == RIGHT or key == UP or key == DOWN or key == W or key == A or key == S or key == D then
		keyControl = true
	elseif key >= KEYSYMS.N_1 and key <= KEYSYMS.N_3 then
		local num = key - KEYSYMS.N_1 + 1
		Spring.SendLuaRulesMsg('spell|' .. tostring(num))
	else
		return
	end
	return true
end

function widget:KeyRelease(key)
	if not (Spring.GetKeyState(A) or Spring.GetKeyState(LEFT) or Spring.GetKeyState(D) or Spring.GetKeyState(RIGHT) or Spring.GetKeyState(W) or Spring.GetKeyState(UP) or Spring.GetKeyState(S) or Spring.GetKeyState(DOWN)) then
		keyControl = false
	end
end

function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == controlledDefID then
		controlledID = unitID
	end
end

function widget:Initialize()
	vsx, vsy = Spring.GetViewGeometry()

	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		widget:UnitCreated(unitID, unitDefID)
	end
end