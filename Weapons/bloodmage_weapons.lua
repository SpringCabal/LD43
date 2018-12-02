local RangedSpell = Weapon:New {
	-- general
	weaponType            = "Cannon",
	impactOnly            = true,
	noSelfDamage          = true,
	range                 = 1000,
	weaponVelocity        = 2000,
	reloadTime            = 3.0,
	tolerance             = 6000,
	 -- collision & avoidance
	avoidFriendly         = true,
	avoidFeature          = true,
	collideFriendly       = true,
	collideFeature        = true,
	impulseFactor = 0,
	 -- targeting & accuracy
	accuracy              = 1.0,
	-- model                 = 'spear.dae',
	 -- soundStart            = [[SpearThrow]],
	soundHit              = [[Hit]],
	damage                = {
		default = 0,
	}
}

local Fireball = RangedSpell:New {
	name = "Fireball",

	impactOnly = false,
	areaOfEffect = 500,
	impulseFactor = 1,
	craterBoost = 100,

    damage = {
        default = 1000,
	},

	customParams = {
		light_radius = 1000,
		light_fade_time = 1,
		light_fade_offset = 10,
	}
}

local Web = RangedSpell:New {
    name = "Web",

    damage = {
        default = 0,
	},

	customParams = {
		light_radius = 1000,
		light_fade_time = 1,
		light_fade_offset = 10,
	}
}

return {
	Fireball = Fireball,
	Web	 	 = Web,
}