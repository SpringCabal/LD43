local House = Unit:New {
	buildPic            = "",
	buildCostMetal        = 0.1, -- used only for power XP calcs
	buildCostEnergy       = 0.1, -- used only for power XP calcs
	buildTime             = 0.1, -- used only for power XP calcs
	maxSlope              = 100,
	canMove             = false,
	maxVelocity         = 0,
--     canGuard            = false,
--     canPatrol           = false,
--     canRepeat           = false,
	 --pushResistant       = true,
	customParams = {
		hscale = 1.0,
		vscale = 1.0,
	},
	 collisionVolumeScales   = '152 60 152',
	collisionVolumeTest     = 1,
	collisionVolumeType     = 'Box',
	footprintX          = 12,
	footprintZ          = 12,
	category            = "HOUSE",
	yardmap =
[[
oooooooooooo
oooooooooooo
oooooooooooo
oooooooooooo
oooooooooooo
oooooooooooo
oooooooooooo
oooooooooooo
oooooooooooo
oooooooooooo
oooooooooooo
oooooooooooo
]],
	mass                = 50,
	minCollisionSpeed   = 1,
	repairable          = false,
	sightDistance       = 800,
	stealth             = true,
	upright             = true,
	name                = "House",
	activateWhenBuilt   = true,
	idletime            = 120, --in simframes
	idleautoheal        = 0,
	autoheal            = 0,
	maxDamage           = 1600,
	onoffable           = true,
	script              = "house.lua",
	objectName 			= "House.dae",

	-- FIXME: uncommenting this freezes Spring
	-- corpse              = [[DEAD_A]],
	featureDefs         = {
		DEAD_A  = {
			blocking         = false,
			featureDead      = [[DEAD_A]],
			footprintX       = 2,
			footprintZ       = 2,
			object           = [[RubbleBrick.dae]],
			smokeTime        = 0,
		},

	},
}


local StreetLight = Unit:New {
	buildPic            = "",
	buildCostMetal        = 0.1, -- used only for power XP calcs
	buildCostEnergy       = 0.1, -- used only for power XP calcs
	buildTime             = 0.1, -- used only for power XP calcs
	maxSlope              = 100,
	canMove             = false,
	maxVelocity         = 0,
--     canGuard            = false,
--     canPatrol           = false,
--     canRepeat           = false,
	 --pushResistant       = true,
	-- customParams = {
	-- 	hscale = 0.5,
	-- 	vscale = 0.2,
	-- },
	 collisionVolumeScales   = '30 50 30',
	collisionVolumeTest     = 1,
	collisionVolumeType     = 'Box',
	footprintX          = 2,
	footprintZ          = 2,
	category            = "HOUSE",
	yardmap =
[[
oooo
oooo
oooo
oooo
]],
	mass                = 50,
	minCollisionSpeed   = 1,
	repairable          = false,
	sightDistance       = 800,
	stealth             = true,
	upright             = true,
	name                = "StreetLight",
	activateWhenBuilt   = true,
	idletime            = 120, --in simframes
	idleautoheal        = 0,
	autoheal            = 0,
	maxDamage           = 100,
	onoffable           = true,
	script              = "streetlight.lua",
	objectName 			= "StreetLight.dae",
}

return {
	house = House,
	StreetLight = StreetLight,
}
