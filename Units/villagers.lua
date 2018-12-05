local units = {}

units.Peasant = Humanoid:New {
	-- General
	name                = "Peasant",
	movementClass       = "Defender",
	objectName 			= "Villager.dae",
	script              = "Swordsman.lua",
	maxDamage           = 500,

	collisionVolumeScales    = '28 34 28',
	selectionVolumeOffsets = [[0 0 0]],
	selectionVolumeScales  = [[20 20 20]],
	selectionVolumeType    = [[ellipsoid]],
	maxVelocity         = 6,

	customParams = {
		hscale = 1,
		vscale = 1,
	},

	iconType = "peasant",

	weapons = {
		{
			name = "DefenderSpear",
		},
		{
			name = "BraveEyes",
		}
	},
}

units.Coward = Humanoid:New {
	-- General
	name                = "Peasant",
	movementClass       = "Defender",
	objectName 			= "VillagerNoWeapon.dae",
	script              = "Coward.lua",
	maxDamage           = 450,
	movestate           = 0,

	collisionVolumeScales    = '28 34 28',
	selectionVolumeOffsets = [[0 0 0]],
	selectionVolumeScales  = [[20 20 20]],
	selectionVolumeType    = [[ellipsoid]],
	maxVelocity         = 5,

	customParams = {
		hscale = 1,
		vscale = 1,
	},

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

	collisionVolumeScales    = '28 34 28',
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
	maxDamage           = 700,
	moveState           = 1, -- 2 results in large swarms
	mass                = 80,

	collisionVolumeScales    = '28 34 28',
	selectionVolumeOffsets = [[0 0 0]],
	selectionVolumeScales  = [[20 20 20]],
	selectionVolumeType    = [[ellipsoid]],
	maxVelocity         = 6,

	customParams = {
		hscale = 0.65,
		vscale = 0.65,
	},

	iconType = "swordsman",

	weapons = {
		{
			name = "DefenderSword",
		},
		{
			name = "BravestEyes",
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
