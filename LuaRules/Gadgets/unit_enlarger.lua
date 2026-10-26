function gadget:GetInfo()
	return {
		name    = "Unit Enlarger",
		desc    = "Scales units physically and graphically",
		author  = "Rafal",
		date    = "May 2015",
		license = "GNU LGPL, v2.1 or later",
		layer   = 20,
		enabled = true
	}
end

--------------------------------------------------------------------------------
-- speedups
--------------------------------------------------------------------------------

CMD_DECREASE_SIZE = 33500
CMD_INCREASE_SIZE = 33501

if (gadgetHandler:IsSyncedCode()) then
--------------------------------------------------------------------------------
-- SYNCED
--------------------------------------------------------------------------------

local spGetUnitDefID      = Spring.GetUnitDefID
local spSetUnitRulesParam = Spring.SetUnitRulesParam
	local LOS_ACCESS = { inlos = true }

local unitData = {}

-------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:Initialize()
	local allUnits = Spring.GetAllUnits()
	for i = 1, #allUnits do
		local unitID = allUnits[i]
		gadget:UnitCreated(unitID, spGetUnitDefID(unitID))
	end
end

function gadget:UnitCreated(unitID, unitDefID)
	local hscale = (UnitDefs[unitDefID].customParams.hscale and tonumber(UnitDefs[unitDefID].customParams.hscale)) or 1
	local vscale = (UnitDefs[unitDefID].customParams.vscale and tonumber(UnitDefs[unitDefID].customParams.vscale)) or 1
	if hscale ~= 1 or vscale ~= 1 then
		spSetUnitRulesParam( unitID, "scale_vertical", vscale, LOS_ACCESS)
		spSetUnitRulesParam( unitID, "scale_horizontal", hscale, LOS_ACCESS)
	end
end
	--------------------------------------------------------------------------------
-- SYNCED
--------------------------------------------------------------------------------
else
--------------------------------------------------------------------------------
-- UNSYNCED
--------------------------------------------------------------------------------

local setUnitLuaDraw = Spring.UnitRendering.SetUnitLuaDraw
local glScale = gl.Scale
local defScales, unitScales = {}, {}
for defID, def in pairs(UnitDefs) do
	local horizontal = tonumber(def.customParams.hscale) or 1
	local vertical = tonumber(def.customParams.vscale) or 1
	if horizontal ~= 1 or vertical ~= 1 then
		defScales[defID] = {horizontal, vertical}
	end
end

function gadget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		self:UnitCreated(unitID, Spring.GetUnitDefID(unitID))
	end
end

function gadget:UnitCreated(unitID, unitDefID)
	local scale = defScales[unitDefID]
	if scale then
		unitScales[unitID] = scale
		setUnitLuaDraw(unitID, true)
	end
end

function gadget:UnitDestroyed(unitID)
	unitScales[unitID] = nil
end

function gadget:DrawUnit(unitID, drawMode)
	local scale = unitScales[unitID]
	if scale then glScale(scale[1], scale[2], scale[1]) end
end

--------------------------------------------------------------------------------
-- UNSYNCED
--------------------------------------------------------------------------------
end
