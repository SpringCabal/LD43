
local OrkSmall = Humanoid:New {
	-- General
	name                = "Ork",
	movementClass       = "Defender",
	objectName          = "OrkSmall.dae",
	script              = "ork.lua",
	maxDamage           = 550,
	mass                = 80,
	footprintX          = 2,
	footprintZ          = 2,

	maxVelocity         = 4,
	acceleration        = 2,
	brakeRate           = 2,
	turnRate            = 1200 / 0.16,

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
	name                = "Ork Brute",
	movementClass       = "BigUnit",
	objectName 			= "OrkBig.dae",
	script              = "ork.lua",
	maxDamage           = 1600,
	mass                = 90,

	maxVelocity         = 4,
	turnRate            = 1000 / 0.16,

	footprintX          = 4,
	footprintZ          = 4,

	collisionVolumeType      = 'ellipsoid',
	collisionVolumeScales    = '64 64 64',
	collisionVolumeOffsets    = '0 0 0',
	selectionVolumeOffsets = [[0 0 0]],
	selectionVolumeScales  = [[96 96 96]],
	selectionVolumeType    = [[ellipsoid]],
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
	name                = "Ork Leader",
	movementClass       = "BigUnit",
	objectName          = "OrkBoss.dae",
	script              = "ork.lua",
	maxDamage           = 12000,
	mass                = 160,

	maxVelocity         = 5.6,
	turnRate            = 800 / 0.16,

	footprintX          = 4,
	footprintZ          = 4,

	collisionVolumeType      = 'ellipsoid',
	collisionVolumeScales    = '220 220 220',
	collisionVolumeOffsets    = '0 -30 0',
	selectionVolumeOffsets = [[0 0 0]],
	selectionVolumeScales  = [[220 220 220]],
	selectionVolumeType    = [[ellipsoid]],
	noChaseCategory = [[HOUSE]],

	iconType = "orkboss",

	customParams = {
		ork = true,
		hscale = 2.5,
		vscale = 2.5,
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
