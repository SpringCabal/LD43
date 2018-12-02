--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name    = "Player Controller",
		desc    = "Unit PC third person view controller.",
		author  = "gajop, GoogleFrog",
		date    = "16 April 2016",
		license = "GNU GPL, v2 or later",
		layer   = 0,
		enabled = true
	}
end


-------------------------------------------------------------------
-- SYNCED
-------------------------------------------------------------------
if gadgetHandler:IsSyncedCode() then

-------------------------------------------------------------------
-------------------------------------------------------------------

local sqrt = math.sqrt

local Vector = Spring.Utilities.Vector

-------------------------------------------------------------------
-------------------------------------------------------------------

local MAX_HEIGHT_CHANGE = 200
local PER_FRAME_CHANGE = 0
local HEIGHT_CHANGE_PER_FRAME = 7

local controlledDefID = UnitDefNames["bloodmage"].id
local controlledID = nil
local moveGoal = nil

local movementMessage

local heightmapChangedFrame = 0
-------------------------------------------------------------------
-------------------------------------------------------------------

local function explode(div,str)
	if (div=='') then return 
		false 
	end
	local pos,arr = 0,{}
	-- for each divider found
	for st,sp in function() return string.find(str,div,pos,true) end do
		table.insert(arr,string.sub(str,pos,st-1)) -- Attach chars left of current divider
		pos = sp + 1 -- Jump past current divider
	end
	table.insert(arr,string.sub(str,pos)) -- Attach chars right of last divider
	return arr
end

-------------------------------------------------------------------
-- Handling unit
-------------------------------------------------------------------

local function GiveClampedMoveGoal(unitID, x, z, radius)
	radius = radius or 16
	local cx, cz = Spring.Utilities.ClampPosition(x, z)
	local cy = Spring.GetGroundHeight(cx, cz)
	local _, height, _ = Spring.GetUnitPosition(unitID)
	Spring.SetUnitMoveGoal(unitID, cx, cy, cz, 10, nil, true)
	moveGoal = moveGoal or {}
	moveGoal.x = cx
	moveGoal.z = cz
end

local function MoveUnit(unitID, x, z)
	if not (unitID and Spring.ValidUnitID(unitID)) then
		return
	end
	if Spring.GetUnitCommands(unitID, 0) > 0 then
		local cmds = Spring.GetUnitCommands(unitID, -1)
		for i = 1, #cmds do
			Spring.GiveOrderToUnit(unitID, CMD.REMOVE, {cmds[i].tag}, {})
		end
	end
	GiveClampedMoveGoal(unitID, x, z, radius)
end

local function ClearMove(unitID)
	Spring.ClearUnitGoal(unitID)
	moveGoal = nil
end

local function UpdateMoveGoal(unitID)
	local x, _, z = Spring.GetUnitPosition(unitID)
	if moveGoal and Vector.DistSq(x, z, moveGoal.x, moveGoal.z) < 1000 then
		ClearMove(unitID)
	end
end

local function AttackTarget(unitID, targetID)
	if not targetID then
		return
	end
	if moveGoal then
		ClearMove(unitID)
	end
	Spring.GiveOrderToUnit(unitID, CMD.ATTACK, {targetID}, {})
end

function gadget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == controlledDefID then
		controlledID = unitID
	end
end

function gadget:UnitDestroyed(unitID)
	if controlledID == unitID then
		controlledID = nil
	end
end

function gadget:GameStart()
	gameStarted = true
end

function gadget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
end

-------------------------------------------------------------------
-- Handling messages
-------------------------------------------------------------------

function HandleLuaMessage(msg)
	if not controlledID or Spring.GetGameRulesParam("game_over") == 1 then
		return
	end
	local msg_table = explode('|', msg)

	local frame = Spring.GetGameFrame()
	local castingFreeze = Spring.GetGameRulesParam("castingFreeze")
	if castingFreeze and castingFreeze > frame then
		return
	end

	if msg_table[1] == 'movement' then
		local x = tonumber(msg_table[2])
		local z = tonumber(msg_table[3])

		if x == 0 and z == 0 then
			movementMessage = false
		else
			movementMessage = {
				frame = frame,
				x = x,
				z = z
			}
		end
	elseif msg_table[1] == 'stop' then
		if controlledID then
			ClearMove(controlledID)
		end
	elseif msg_table[1] == 'attack' then
		local targetID = tonumber(msg_table[2])
		AttackTarget(controlledID, targetID)
	elseif msg_table[1] == 'spell' then
		table.remove(msg_table, 1)
		for i = 1, #msg_table do
			msg_table[i] = tonumber(msg_table[i])
		end
		GG.UseSpell(controlledID, unpack(msg_table))
	end
end

function gadget:RecvLuaMsg(msg)
	HandleLuaMessage(msg)
end

function gadget:GameFrame(frame)
	if controlledID then
		local x, y, z = Spring.GetUnitPosition(controlledID)
		if (movementMessage and movementMessage.frame + 2 > frame) then
			MoveUnit(controlledID, movementMessage.x, movementMessage.z)
		end
		UpdateMoveGoal(controlledID)
	end
end


-------------------------------------------------------------------
-- UNSYNCED
-------------------------------------------------------------------
else
-------------------------------------------------------------------

  return

-------------------------------------------------------------------
end

