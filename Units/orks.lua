
local OrkSmall = Humanoid:New {
	-- General
	name                = "OrkSmall",
	movementClass       = "Defender",
	objectName          = "OrkSmall.dae",
	script              = "ork.lua",
	maxDamage           = 500,
	mass                = 80,
	footprintX          = 2,
	footprintZ          = 2,

	maxVelocity         = 4,
	acceleration        = 2,
	brakeRate           = 2,
	turnRate            = 1600 / 0.16,

	selectionVolumeOffsets = [[0 0 0]],
	selectionVolumeScales  = [[50 50 50]],
	selectionVolumeType    = [[ellipsoid]],
	noChaseCategory = [[HOUSE]],

	customParams = {
		ork = true,
		hscale = 1,
		vscale = 1,
	},

	iconType = "orksmall",

	weapons = {
		{
			name = "Bite",
			onlyTargetCategory = [[INFANTRY]],
		},
		{
			name = "Firebomb",
			onlyTargetCategory = [[HOUSE]],
		},
	},
}

local OrkBig = Humanoid:New {
	-- General
	name                = "OrkBig",
	movementClass       = "BigUnit",
	objectName 			= "OrkBig.dae",
	script              = "ork.lua",
	maxDamage           = 1800,
	-- mass                = 50, -- does this even matter?

	maxVelocity         = 4,

	footprintX          = 4,
	footprintZ          = 4,

	collisionVolumeType      = 'ellipsoid',
	collisionVolumeScales    = '96 96 96',
	noChaseCategory = [[HOUSE]],

	iconType = "orkbig",

	customParams = {
		ork = true,
		hscale = 1,
		vscale = 1,
	},

	weapons = {
		{
			name = "BigOrkAxe",
			onlyTargetCategory = [[INFANTRY]],
		},
		{
			name = "Firebomb",
			onlyTargetCategory = [[HOUSE]],
		},
	},
}

local OrkBoss = Humanoid:New {
	-- General
	name                = "OrkBoss",
	movementClass       = "BigUnit",
	objectName          = "OrkBoss.dae",
	script              = "ork.lua",
	maxDamage           = 10000,
	-- mass                = 50, -- does this even matter?

	maxVelocity         = 5.1,

	footprintX          = 4,
	footprintZ          = 4,

	collisionVolumeType      = 'ellipsoid',
	collisionVolumeScales    = '220 220 220',
	noChaseCategory = [[HOUSE]],

	iconType = "orkboss",

	customParams = {
		ork = true,
	},

	weapons = {
		{
			name = "BossOrkAxe",
			onlyTargetCategory = [[INFANTRY]],
		},
		{
			name = "Firebomb",
			onlyTargetCategory = [[HOUSE]],
		},
	},
}


return {
	OrkSmall    = OrkSmall,
	OrkBig		= OrkBig,
	OrkBoss		= OrkBoss
}
