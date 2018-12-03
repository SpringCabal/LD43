local weapons = {}

weapons.MeleeWeapon = Weapon:New {
	-- general
	weaponType            = "Melee",
	impactOnly            = true,
	noSelfDamage          = true,
	range                 = 60,
	weaponVelocity        = 1000,
	reloadTime            = 1.0,
	tolerance             = 6000,
	impulseFactor           = 0,
	 -- collision & avoidance
	avoidFriendly         = false,
	avoidFeature          = false,
	 collideFriendly       = false,
	collideFeature        = false,
	targetBorder            = true,
	 -- targeting & accuracy
	turret               = true,
	accuracy              = 0.9,
	 -- soundStart            = [[SpearThrow]],
	soundHit              = [[Hit]],
	 damage                = {
		default = 100,
	}
}

weapons.FancyMeleeWeapon = Weapon:New {
	name                    = [[Auto Particle Beam]],
	beamDecay               = 0.85,
	beamTime                = 0.1,
	beamttl                 = 1/30,
	coreThickness           = 0.5,
	craterBoost             = 0,
	craterMult              = 0,

	avoidFriendly         = false,
	avoidFeature          = false,
	collideFriendly       = false,
	collideFeature        = false,
	
	damage                  = {
		default = 100
	},

	impactOnly              = true,
	fireStarter             = 100,
	impulseFactor           = 0,
	laserFlareSize          = 1,
	minIntensity            = 1,
	rgbColor                = [[1 0 0]],
	soundStartVolume        = 6,
	thickness               = 1, -- Make this 1 for visibility (debug purposes)
	tolerance               = 8000,
	turret                  = true,
	weaponType              = [[BeamLaser]],
	
	-- New
	range                   = 85,
	reloadtime              = 0.6,
}

return weapons
