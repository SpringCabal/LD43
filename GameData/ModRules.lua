	--Wiki: http://springrts.com/wiki/Modrules.lua

local modRules = {
	movement = {
		allowPushingEnemyUnits    = true,
		allowCrushingAlliedUnits  = false,
		allowUnitCollisionDamage  = true,
		allowUnitCollisionOverlap = false,
		allowGroundUnitGravity    = false,
		allowDirectionalPathing   = true,
	},
	system = {
		pathFinderSystem = 0, -- legacy
	},
  sensors = {
    los = {
      losMipLevel = 6,  -- defaults to 1
      losMul      = 6,  -- defaults to 1
      airMipLevel = 6,  -- defaults to 2
    },
  },
}

return modRules
