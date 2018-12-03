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
		default = 120,
	}
}

weapons.DefenderSword = MeleeWeapon:New {
	name                  = "DefenderSword",
	range                 = 60,
	reloadTime            = 1.2,
	-- targeting & accuracy
	accuracy              = 0.9,
	 -- soundStart            = [[SpearThrow]],
	soundStart            = "swordswing",
	 damage                = {
		default = 130,
	}
}

weapons.DefenderSpear = MeleeWeapon:New {
	name                  = "DefenderSpear",
	range                 = 100,
	reloadTime            = 1.5,
	-- targeting & accuracy
	accuracy              = 0.9,
	 -- soundStart            = [[SpearThrow]],
	soundStart            = "stickswing",
	 damage                = {
		default = 30,
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
	soundStart            = "runawayyell",
	 damage                = {
		default = 0,
	}
}

weapons.Staff = FancyMeleeWeapon:New {
	name                  = "Staff",
	reloadTime            = 0.8,
	areaOfEffect          = 120,
	soundStart            = "staffswing",
	soundHit              = "staffhit",
	 damage                = {
		default = 275,
	}
}

return weapons
