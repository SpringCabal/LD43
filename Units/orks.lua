
local OrkSmall = Humanoid:New {
	-- General
	name                = "OrkSmall",
	movementClass       = "Defender",
	objectName          = "OrkSmall.dae",
	script              = "ork.lua",
	maxDamage           = 500,
	-- mass                = 50, -- does this even matter?
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
	movementClass       = "Defender",
	objectName 			= "OrkBig.dae",
	script              = "ork.lua",
	maxDamage           = 1000,
	-- mass                = 50, -- does this even matter?

	maxVelocity         = 4,

	footprintX          = 2,
	footprintZ          = 2,

	collisionVolumeType      = 'CylY',
	collisionVolumeScales    = '102.0 69.0 102.0',

	iconType = "orkbig",

	customParams = {
		ork = true,
	},

	weapons = {
		{
			name = "BigOrkAxe",
			onlyTargetCategory = [[INFANTRY]],
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

	footprintX          = 2,
	footprintZ          = 2,

	collisionVolumeType      = 'CylY',
	collisionVolumeScales    = '320 235 320',

	iconType = "orkboss",

	customParams = {
		ork = true,
	},

	weapons = {
		{
			name = "BossOrkAxe",
			onlyTargetCategory = [[INFANTRY]],
		}
	},
}


return {
	OrkSmall    = OrkSmall,
	OrkBig		= OrkBig,
	OrkBoss		= OrkBoss
}
