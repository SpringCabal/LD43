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
	avoidFriendly         = true,
	avoidFeature          = true,
	 collideFriendly       = false,
	collideFeature        = false,
	targetBorder            = true,
	 -- targeting & accuracy
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
	beamTime                = 0.5,
	beamttl                 = 1/30,
	coreThickness           = 0.5,
	craterBoost             = 0,
	craterMult              = 0,

	damage                  = {
		default = 100
	},

	fireStarter             = 100,
	impulseFactor           = 0,
	laserFlareSize          = 1,
	minIntensity            = 1,
	rgbColor                = [[1 0 0]],
	soundStartVolume        = 6,
	thickness               = 1,
	tolerance               = 8192,
	turret                  = false,
	weaponType              = [[BeamLaser]],
	
	-- New
	range                   = 70,
	reloadtime              = 0.6,
	targetBorder            = true,
}

return weapons
