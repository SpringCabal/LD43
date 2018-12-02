function gadget:GetInfo()
	return {
		name    = "Structure Typemap",
		desc    = "Sets structures to have unpathable terrain under them.",
		author  = "GoogleFrog",
		date    = "2 December 2018",
		license = "GNU LGPL, v2.1 or later",
		layer   = 0,
		enabled = true
	}
end
	if (gadgetHandler:IsSyncedCode()) then
--------------------------------------------------------------------------------
-- SYNCED
--------------------------------------------------------------------------------

local spGetUnitDefID      = Spring.GetUnitDefID
local IMPASSIBLE_TERRAIN = 137 -- Hope that this does not conflict with any maps

local PLAYER_FUDGE = 8

-------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local function GetUnitExtents(unitID, ud)
	local ux,_,uz = Spring.GetUnitPosition(unitID, true)
	local face = Spring.GetUnitBuildFacing(unitID)
	local xsize = (ud.xsize)*4
	local zsize = (ud.ysize or ud.zsize)*4
	local minx, minz, maxx, maxz
	if ((face == 0) or (face == 2)) then
		if xsize%16 == 0 then
			ux = math.floor((ux+8)/16)*16
		else
			ux = math.floor(ux/16)*16+8
		end
	if zsize%16 == 0 then
			uz = math.floor((uz+8)/16)*16
		else
			uz = math.floor(uz/16)*16+8
		end
		minx = ux - xsize
		minz = uz - zsize
		maxx = ux + xsize
		maxz = uz + zsize
	else
		if xsize%16 == 0 then
			uz = math.floor((uz+8)/16)*16
		else
			uz = math.floor(uz/16)*16+8
		end
	if zsize%16 == 0 then
			ux = math.floor((ux+8)/16)*16
		else
			ux = math.floor(ux/16)*16+8
		end
	minx = ux - zsize
		minz = uz - xsize
		maxx = ux + zsize
		maxz = uz + xsize
	end
	return minx - 8, minz - 8, maxx, maxz
end

local function SetTypemapSquare(minx, minz, maxx, maxz, value)
	for x = minx, maxx, 8 do
		for z = minz, maxz, 8 do
			if value == IMPASSIBLE_TERRAIN and (x < minx + PLAYER_FUDGE or x > maxx - PLAYER_FUDGE or z < minz + PLAYER_FUDGE or z > maxz - PLAYER_FUDGE) then
	Spring.SetMapSquareTerrainType(x, z, value)
			else
	Spring.SetMapSquareTerrainType(x, z, value)
			end
		end
	end
end

-------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:UnitCreated(unitID, unitDefID)
	local ud = UnitDefs[unitDefID]
	if not ud.isImmobile then
		return
	end

	local minx, minz, maxx, maxz = GetUnitExtents(unitID, ud)
	SetTypemapSquare(minx, minz, maxx, maxz, IMPASSIBLE_TERRAIN)
end

function gadget:UnitDestroyed(unitID, unitDefID)
	local ud = UnitDefs[unitDefID]
	if not ud.isImmobile then
		return
	end

	local minx, minz, maxx, maxz = GetUnitExtents(unitID, ud)
	SetTypemapSquare(minx, minz, maxx, maxz, 0)
end

function gadget:Initialize()
	Spring.SetTerrainTypeData(IMPASSIBLE_TERRAIN, 0, 0, 0, 0)
	Spring.SetTerrainTypeData(IMPASSIBLE_TERRAIN + 1, 0.5, 1, 1, 1)
	local allUnits = Spring.GetAllUnits()
	for i = 1, #allUnits do
		local unitID = allUnits[i]
		gadget:UnitCreated(unitID, spGetUnitDefID(unitID))
	end
end

end
