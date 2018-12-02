local Peasant = Humanoid:New {
	-- General
	name                = "Peasant",
	movementClass       = "Defender",
	objectName 			= "HumanWithStick.dae",
	script              = "defender.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?

	collisionVolumeScales    = '37 43 37',

	weapons = {
		{
			name = "DefenderSpear",
		}
	},
}

local Crossbowman = Humanoid:New {

	-- General
	name                = "Crossbowman",
	movementClass       = "Defender",
	objectName 			= "HumanCrossbow.dae",
	script              = "defender.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?

	collisionVolumeScales    = '37 64 37',

	weapons = {
		{
			name = "CrossBow",
		}
	},
}

local Builder = Humanoid:New {

	-- General
	name                = "Builder",
	movementClass       = "Defender",
	objectName 			= "HumanCrossbow.dae",
	script              = "defender.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?
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

local Swordsman = Humanoid:New {
	-- General
	name                = "Swordsman",
	movementClass       = "Defender",
	objectName 			= "Swordsman.dae",
	script              = "defender.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?

	collisionVolumeScales    = '37 64 37',
	maxVelocity         = 3,
	footprintX 			= 3,
	footprintZ 			= 3,

	weapons = {
		{
			name = "DefenderSword",
		}
	},
}

return {
	Swordsman   = Swordsman,
	Crossbowman = Crossbowman,
	Peasant     = Peasant,
	Builder     = Builder,
}
