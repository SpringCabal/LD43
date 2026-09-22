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
	-- Keep the projectile in the Cannon path so its visual can use the
	-- game's round fireball texture. DGun is a hard-coded streak/trail and
	-- reads the engine dgun texture, which is the missile-like result seen in
	-- the WASM build.
	weaponType = "Cannon",
	texture1 = "fireball_rot",
	size = 22,

	impactOnly = false,
	areaOfEffect = 500,
	craterAreaOfEffect = 500,
	impulseFactor = 0.6,
	explosionSpeed = 35,
	craterBoost = 100,
	soundStart             = "firebombfire",
	soundHit               = "fireballhit",
	soundStartVolume       = 10,
	edgeEffectiveness = 0.9,

    damage = {
        default = 1600,
	},

	customParams = {
		light_radius = 350,
		light_fade_time = 1,
		light_fade_offset = 10,
	},

	cegTag = "fireball_ball"
}

return {
	Fireball = Fireball,
}
