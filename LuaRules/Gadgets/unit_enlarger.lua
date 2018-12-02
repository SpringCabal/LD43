function gadget:GetInfo()
	return {
		name    = "Unit Enlarger",
		desc    = "Scales units physically and graphically",
		author  = "Rafal",
		date    = "May 2015",
		license = "GNU LGPL, v2.1 or later",
		layer   = 0,
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
	if UnitDefs[unitDefID].customParams.hscale or UnitDefs[unitDefID].customParams.vscale then
		spSetUnitRulesParam( unitID, "scale_vertical", tonumber(UnitDefs[unitDefID].customParams.vscale) or 1, LOS_ACCESS)
		spSetUnitRulesParam( unitID, "scale_horizontal", tonumber(UnitDefs[unitDefID].customParams.hscale) or 1, LOS_ACCESS)
		SendToUnsynced( "Enlarger_SetUnitLuaDraw", unitID, true )
	end
end


--------------------------------------------------------------------------------
-- SYNCED
--------------------------------------------------------------------------------
else
--------------------------------------------------------------------------------
-- UNSYNCED
--------------------------------------------------------------------------------

local spurSetUnitLuaDraw  = Spring.UnitRendering.SetUnitLuaDraw
local spGetUnitRulesParam = Spring.GetUnitRulesParam
local spGetUnitPosition   = Spring.GetUnitPosition

local glTranslate = gl.Translate
local glScale     = gl.Scale

--------------------------------------------------------------------------------

local function SetUnitLuaDraw(_, unitID, enabled)
	spurSetUnitLuaDraw (unitID, enabled)
end

function gadget:Initialize()
	Spring.UnitRendering.SetUnitLuaDraw (1, true);
	gadgetHandler:AddSyncAction("Enlarger_SetUnitLuaDraw", SetUnitLuaDraw)
end

function gadget:Shutdown()
	gadgetHandler.RemoveSyncAction("Enlarger_SetUnitLuaDraw")
end


function gadget:DrawUnit(unitID, drawMode)
	local vScale = spGetUnitRulesParam( unitID, "scale_vertical");
	local hScale = spGetUnitRulesParam( unitID, "scale_horizontal");
	if not (vScale or hScale) then
		return
	end
	
	--local bx, by, bz = spGetUnitPosition(unitID)
	--glTranslate( -bx, -by, -bz )
	glScale(hScale, vScale, hScale)
	--glTranslate( bx, by, bz )
end

--------------------------------------------------------------------------------
-- UNSYNCED
--------------------------------------------------------------------------------
end
