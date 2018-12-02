local units = {}

units.Peasant = Humanoid:New {
	-- General
	name                = "Peasant",
	movementClass       = "Defender",
	objectName 			= "HumanWithStick.dae",
	script              = "defender.lua",
	maxDamage           = 350,

	collisionVolumeScales    = '37 43 37',
	maxVelocity         = 3,

	weapons = {
		{
			name = "DefenderSpear",
		}
	},
}

units.Crossbowman = Humanoid:New {
	-- General
	name                = "Crossbowman",
	movementClass       = "Defender",
	objectName 			= "HumanCrossbow.dae",
	script              = "crossbowman.lua",
	maxDamage           = 400,

	collisionVolumeScales    = '37 64 37',
	maxVelocity         = 3,

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
	maxDamage           = 600,
	moveState           = 2,

	collisionVolumeScales    = '37 64 37',
	maxVelocity         = 3,
	footprintX 			= 2,
	footprintZ 			= 2,

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
