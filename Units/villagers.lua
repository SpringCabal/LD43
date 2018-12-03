local units = {}

units.Peasant = Humanoid:New {
	-- General
	name                = "Peasant",
	movementClass       = "Defender",
	objectName 			= "HumanWithStick.dae",
	script              = "Swordsman.lua",
	maxDamage           = 600,

	collisionVolumeScales    = '37 43 37',
	selectionVolumeOffsets = [[0 0 0]],
	selectionVolumeScales  = [[20 20 20]],
	selectionVolumeType    = [[ellipsoid]],
	maxVelocity         = 6,

	iconType = "peasant",

	weapons = {
		{
			name = "DefenderSpear",
		}
	},
}

units.Coward = Humanoid:New {
	-- General
	name                = "Coward",
	movementClass       = "Defender",
	objectName 			= "HumanWithStick.dae",
	script              = "Coward.lua",
	maxDamage           = 600,
	movestate           = 0,

	collisionVolumeScales    = '37 43 37',
	selectionVolumeOffsets = [[0 0 0]],
	selectionVolumeScales  = [[20 20 20]],
	selectionVolumeType    = [[ellipsoid]],
	maxVelocity         = 6,

	iconType = "peasant",

	weapons = {
		{
			name = "CowardsEyes",
		}
	},
}

units.Crossbowman = Humanoid:New {
	-- General
	name                = "Crossbowman",
	movementClass       = "Defender",
	objectName 			= "HumanCrossbow.dae",
	script              = "crossbowman.lua",
	maxDamage           = 600,

	collisionVolumeScales    = '37 46 37',
	selectionVolumeOffsets = [[0 0 0]],
	selectionVolumeScales  = [[20 20 20]],
	selectionVolumeType    = [[ellipsoid]],
	maxVelocity         = 6,

	iconType = "crossbowman",

	customParams = {
		hscale = 1,
		vscale = 1,
	},

	weapons = {
		{
			name = "CrossBow",
		}
	},
}

units.Swordsman = Humanoid:New {
	-- General
	name                = "Swordsman",
	movementClass       = "Defender",
	objectName 			= "Swordsman.dae",
	script              = "swordsman.lua",
	maxDamage           = 800,
	moveState           = 2, -- 2 results in large swarms
	mass                = 80,

	collisionVolumeScales    = '23 34 23',
	selectionVolumeOffsets = [[0 0 0]],
	selectionVolumeScales  = [[20 20 20]],
	selectionVolumeType    = [[ellipsoid]],
	maxVelocity         = 6,
	footprintX 			= 2,
	footprintZ 			= 2,

	customParams = {
		hscale = 0.65,
		vscale = 0.65,
	},

	iconType = "swordsman",

	weapons = {
		{
			name = "DefenderSword",
		}
	},
}

units.Builder = Humanoid:New {
	-- General
	name                = "Builder",
	movementClass       = "Defender",
	objectName 			= "HumanCrossbow.dae",
	script              = "defender.lua",
	maxDamage           = 1600,
	builder = true,
	workerTime = 10000,
	buildOptions = {"house"},
	buildDistance = 1000000,
	terraformSpeed = 100000000,

	collisionVolumeScales    = '37 64 37',

	weapons = {
		{
			name = "CrossBow",
		}
	},
}

return units
