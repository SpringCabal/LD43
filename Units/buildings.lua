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
		hscale = 0.5,
		vscale = 0.2,
	},
	 collisionVolumeScales   = '192 50 192',
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
}

return {
	house = House
}
