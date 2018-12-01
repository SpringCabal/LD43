local MeleeWeapon = Weapon:New {
	-- general
	weaponType            = "Melee",
	impactOnly            = true,
	noSelfDamage          = true,
	range                 = 100,
	weaponVelocity        = 1000,
	reloadTime            = 1.0,
	tolerance             = 6000,
	 -- collision & avoidance
	avoidFriendly         = true,
	avoidFeature          = true,
	 collideFriendly       = false,
	collideFeature        = false,
	 -- targeting & accuracy
	accuracy              = 0.9,
	 -- soundStart            = [[SpearThrow]],
	soundHit              = [[Hit]],
	 damage                = {
		default = 100,
	}
}

return {
	MeleeWeapon = MeleeWeapon
}