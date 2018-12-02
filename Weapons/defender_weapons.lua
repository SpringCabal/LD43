-- Comments on the right are my grievances with the format
local weapons = {}

weapons.CrossBow = Weapon:New {
	-- general
	weaponType            = "Cannon", -- there's a default, but honestly, why?
	name                  = "Spear",
	impactOnly            = true,
	noSelfDamage          = true,
	range                 = 1000,     -- bad defaults (only 10.0)
	weaponVelocity        = 2000,     -- default velocity is 0, wtf?
	reloadTime            = 3.0,
	tolerance             = 6000,
	 -- collision & avoidance
	avoidFriendly         = false,
	avoidFeature          = false,
	 collideFriendly       = false,
	collideFeature        = false,
	 impulseFactor = 0,
	 -- targeting & accuracy
	accuracy              = 0.1,
	-- model                 = 'spear.dae',
	 -- soundStart            = [[SpearThrow]],
	soundHit              = [[Hit]],
	 damage                = {
		default = 100,
	}
}

weapons.DefenderSword = MeleeWeapon:New {
	name                  = "DefenderSword",
	range                 = 100,
	reloadTime            = 2.0,
	-- targeting & accuracy
	accuracy              = 0.9,
	 -- soundStart            = [[SpearThrow]],
	soundHit              = [[Hit]],
	 damage                = {
		default = 100,
	}
}

weapons.DefenderSpear = MeleeWeapon:New {
	name                  = "DefenderSpear",
	range                 = 100,
	reloadTime            = 1.5,
	-- targeting & accuracy
	accuracy              = 0.9,
	 -- soundStart            = [[SpearThrow]],
	soundHit              = [[Hit]],
	 damage                = {
		default = 30,
	}
}

weapons.Staff = MeleeWeapon:New {
	name                  = "Staff",
	reloadTime            = 0.8,
	 damage                = {
		default = 200,
	}
}

return weapons
