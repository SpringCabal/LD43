local RangedSpell = Weapon:New {
	-- general
	weaponType            = "Cannon",
	impactOnly            = true,
	noSelfDamage          = true,
	range                 = 1000,
	weaponVelocity        = 500,
	reloadTime            = 3.0,
	tolerance             = 6000,
	 -- collision & avoidance
	avoidFriendly         = false,
	avoidFeature          = false,
	collideFriendly       = false,
	collideFeature        = false,
	impulseFactor         = 0,
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
	areaOfEffect = 300,
	craterAreaOfEffect = 350,
	impulseFactor = 0.2,
	explosionSpeed = 30,
	craterBoost = 100,

    damage = {
        default = 1000,
	},

	customParams = {
		light_radius = 350,
		light_fade_time = 1,
		light_fade_offset = 10,
	}
}

local Web = RangedSpell:New {
    name = "Web",

    damage = {
        default = 0,
	},

	rgbColor = {
		0.3, 0.0, 0.6,
	},

	customParams = {
		light_radius = 100,
		light_fade_time = 1,
		light_fade_offset = 10,
	}
}

return {
	Fireball = Fireball,
	Web	 	 = Web,
}