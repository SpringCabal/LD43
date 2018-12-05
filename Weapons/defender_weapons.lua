-- Comments on the right are my grievances with the format
local weapons = {}

weapons.CrossBow = Weapon:New {
	-- general
	weaponType            = "Cannon", -- there's a default, but honestly, why?
	name                  = "Spear",
	impactOnly            = true,
	noSelfDamage          = true,
	range                 = 800,     -- bad defaults (only 10.0)
	weaponVelocity        = 2000,     -- default velocity is 0, wtf?
	reloadTime            = 3.0,
	tolerance             = 6000,
	 -- collision & avoidance
	avoidFriendly         = true,
	avoidFeature          = false,
	 collideFriendly       = false,
	collideFeature        = false,
	 impulseFactor = 0,
	 -- targeting & accuracy
	accuracy              = 0.1,
	-- model                 = 'spear.dae',
	 -- soundStart            = [[SpearThrow]],
	soundStart             = "crossbowfire",
	soundHit               = "crossbowhit",
	 damage                = {
		default = 70,
	}
}

weapons.DefenderSword = FancyMeleeWeapon:New {
	name                  = "DefenderSword",
	range                 = 80,
	reloadTime            = 1.2,
	-- targeting & accuracy
	accuracy              = 0.9,
	 -- soundStart            = [[SpearThrow]],
	soundStart            = "swordswing",
	 damage                = {
		default = 90,
	}
}

weapons.DefenderSpear = FancyMeleeWeapon:New {
	name                  = "DefenderSpear",
	range                 = 80,
	reloadTime            = 1.4,
	-- targeting & accuracy
	accuracy              = 0.9,
	 -- soundStart            = [[SpearThrow]],
	soundStart            = "stickswing",
	 damage                = {
		default = 45,
	}
}

weapons.CowardsEyes = MeleeWeapon:New {
	name                  = "CowardsEyes",
	range                 = 350,
	reloadTime            = 3,
	-- targeting & accuracy
	accuracy              = 0.9,
	turret                = true,
	 -- soundStart            = [[SpearThrow]],
	 damage                = {
		default = 0,
	}
}

weapons.Staff = FancyMeleeWeapon:New {
	name                  = "Staff",
	reloadTime            = 0.8,
	areaOfEffect          = 120,
	noExplode             = true,
	impactOnly            = false,
	soundStart            = "staffswing",
	soundHit              = "staffhit",
	soundHitVolume        = 8,
	edgeEffectiveness = 0.9,
	 damage                = {
		default = 300,
	}
}

return weapons
