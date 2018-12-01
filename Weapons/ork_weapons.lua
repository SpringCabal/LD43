-- Comments on the right are my grievances with the format

local Axe = Weapon:New {
    -- general
    weaponType            = "Melee", -- there's a default, but honestly, why?
    name                  = "Axe",
    impactOnly            = true,
    noSelfDamage          = true,
    range                 = 700,
    weaponVelocity        = 1000,
    reloadTime            = 3.0,
    tolerance             = 6000,

    -- collision & avoidance
    avoidFriendly         = true,
    avoidFeature          = true,

    collideFriendly       = false,
    collideFeature        = false,

    -- targeting & accuracy
    accuracy              = 0.9,
    -- model                 = 'spear.dae',

    soundStart            = [[SpearThrow]],
    soundHit              = [[Hit]],

    damage                = {
        default = 1000,
    }
}

return {
	Axe = Axe
}
