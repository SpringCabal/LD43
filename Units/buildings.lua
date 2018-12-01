local House = Unit:New {
	buildPic            = "",

    --buildCostMetal        = 65, -- used only for power XP calcs
    canMove             = false,
    maxVelocity         = 0,
--     canGuard            = false,
--     canPatrol           = false,
--     canRepeat           = false,

    --pushResistant       = true,

    collisionVolumeScales   = '37 40 37',
    collisionVolumeTest     = 1,
    collisionVolumeType     = 'CylY',
    footprintX          = 6,
    footprintZ          = 6,
    yardmap = "oooooooooooooooooooooooooooooooooooo",

    mass                = 50,
    minCollisionSpeed   = 1,

    repairable          = false,
    sightDistance       = 800,


    stealth             = true,
    upright             = true,


    name                = "House",
    activateWhenBuilt   = true,
    customParams = {   },

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
