function gadget:GetInfo()
	return {
		name    = "Unit Script Support API",
		desc    = "Bridges the two things the Lua unit scripts need that only Lua can provide.",
		author  = "SpringCabal",
		license = "GNU GPL, v2 or later",
		layer   = -10,
		enabled = true,
	}
end

--------------------------------------------------------------------------------
-- Why this gadget exists
--
-- Everything the port needs from the engine it now calls directly from WASM.
-- Two things remain that a WASM rules module cannot do:
--   * sound playback, which is unsynced-only and has to cross the sync boundary;
--   * GG.Attributes, which the Lua unit scripts call but which lives inside the
--     WASM module, where Lua cannot reach.  Scripts only use it to freeze a
--     corpse while its death animation plays, so that is all this implements.
--------------------------------------------------------------------------------

local PAUSED_RELOAD = 100000

if gadgetHandler:IsSyncedCode() then
	local spGetUnitDefID = Spring.GetUnitDefID
	local spGetUnitWeaponState = Spring.GetUnitWeaponState
	local spSetUnitWeaponState = Spring.SetUnitWeaponState

	-- Sound playback is unsynced-only, so hand it over the sync boundary.
	function GG.PlaySound(sound, volume, x, y, z)
		SendToUnsynced("playSound", sound, volume, x, y, z)
	end

	local function FreezeWeapon(unitID, factor)
		if factor > 0 then
			return
		end
		local reload = spGetUnitWeaponState(unitID, 1, "reloadTime")
		if reload and reload > 0 then
			spSetUnitWeaponState(unitID, 1, "reloadTime", PAUSED_RELOAD)
		end
	end

	-- The synced module has already forgotten a unit by the time its script
	-- freezes the corpse, so nothing here has to be undone later.
	GG.Attributes = {
		AddEffect = function (unitID, _name, modifier)
			local move = modifier.move or 1
			FreezeWeapon(unitID, modifier.reload or 1)
			local unitDefID = spGetUnitDefID(unitID)
			local unitDef = unitDefID and UnitDefs[unitDefID]
			if unitDef and unitDef.speed and unitDef.speed > 0 then
				local speed = unitDef.speed * move
				Spring.MoveCtrl.SetGroundMoveTypeData(unitID, {
					maxSpeed = speed,
					maxWantedSpeed = speed,
				})
				if move <= 0 then
					Spring.SetUnitVelocity(unitID, 0, 0, 0)
				end
			end
		end,
	}
else
	--------------------------------------------------------------------
	-- Unsynced side: the receiving end of the sound relay.
	--------------------------------------------------------------------
	local function PlaySound(_, sound, volume, x, y, z)
		Spring.PlaySoundFile(sound, volume, x, y, z)
	end

	function gadget:Initialize()
		gadgetHandler:AddSyncAction("playSound", PlaySound)
	end

	function gadget:Shutdown()
		gadgetHandler:RemoveSyncAction("playSound")
	end
end
