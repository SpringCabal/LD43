-- Wiki: http://springrts.com/wiki/Movedefs.lua
-- See also; http://springrts.com/wiki/Units-UnitDefs#Tag:movementClass

local moveDefs  =    {
	{
		name             = "Defender",
		footprintX       = 2,
		footprintZ       = 2,
		maxWaterDepth    = 10,
		maxSlope         = 60,
		crushStrength    = 5,
		speedModClass    = 1,
		heatmapping      = false,
		allowRawMovement = true,
	},
	{
		name             = "Player",
		footprintX       = 2,
		footprintZ       = 2,
		maxWaterDepth    = 10,
		maxSlope         = 60,
		crushStrength    = 5,
		speedModClass    = 0, -- Important
		heatmapping      = false,
		allowRawMovement = true,
	},
}

return moveDefs