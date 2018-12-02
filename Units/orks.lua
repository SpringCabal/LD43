
local OrkSmall = Humanoid:New {
	-- General
	name                = "OrkSmall",
	movementClass       = "Defender",
	objectName 			= "OrkSmall.dae",
	script              = "ork.lua",
	maxDamage           = 500,
	-- mass                = 50, -- does this even matter?
	footprintX 			= 2,
	footprintZ 			= 2,
	maxVelocity         = 6,

	customParams = {
		ork = true,
	},

	weapons = {
		{
			name = "Bite",
		}
	},
}

local OrkBig = Humanoid:New {
	-- General
	name                = "OrkBig",
	movementClass       = "Defender",
	objectName 			= "OrkBig.dae",
	script              = "ork.lua",
	maxDamage           = 1000,
	-- mass                = 50, -- does this even matter?

	maxVelocity         = 4,

	footprintX 			= 8,
	footprintZ 			= 8,

	collisionVolumeType      = 'CylY',
	collisionVolumeScales    = '149 119 149',

	customParams = {
		ork = true,
	},

	weapons = {
		{
			name = "BigOrkAxe",
		}
	},
}

local OrkBoss = Humanoid:New {
	-- General
	name                = "OrkBoss",
	movementClass       = "Defender",
	objectName 			= "OrkBoss.dae",
	script              = "ork.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?

	maxVelocity         = 5.1,

	footprintX 			= 12,
	footprintZ 			= 12,

	collisionVolumeType      = 'CylY',
	collisionVolumeScales    = '320 235 320',

	customParams = {
		ork = true,
	},

	weapons = {
		{
			name = "BossOrkAxe",
		}
	},
}


return {
	OrkSmall    = OrkSmall,
	OrkBig		= OrkBig,
	OrkBoss		= OrkBoss
}
