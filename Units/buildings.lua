local House = Unit:New {
	buildPic            = "",
	 --buildCostMetal        = 65, -- used only for power XP calcs
	canMove             = false,
	maxVelocity         = 0,
--     canGuard            = false,
--     canPatrol           = false,
--     canRepeat           = false,
	 --pushResistant       = true,
	customParams = {
		hscale = 0.4,
		vscale = 0.22,
	},
	 collisionVolumeScales   = '37 40 37',
	collisionVolumeTest     = 1,
	collisionVolumeType     = 'Box',
	footprintX          = 10,
	footprintZ          = 10,
	yardmap =
[[
oooooooooo
oooooooooo
oooooooooo
oooooooooo
oooooooooo
oooooooooo
oooooooooo
oooooooooo
oooooooooo
oooooooooo
]],
	 mass                = 50,
	minCollisionSpeed   = 1,
	 repairable          = false,
	sightDistance       = 800,
	  stealth             = true,
	upright             = true,
	  name                = "House",
	activateWhenBuilt   = true,
	 idletime					= 120, --in simframes
	idleautoheal 				= 50,
	autoheal 					= 1,
	 maxDamage           = 1600,
	onoffable           = true,
	script              = "house.lua",
	objectName 			= "House.dae",
}

return {
	house = House
}
