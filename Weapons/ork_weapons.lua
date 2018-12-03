local BigOrkAxe = FancyMeleeWeapon:New {
	name                  = "BigOrkAxe",
	range                 = 120,
	reloadTime            = 2.5,
	impactOnly            = true,
	-- targeting & accuracy
	accuracy              = 0.9,
	 -- soundStart            = [[SpearThrow]],
	soundStart             = "bigorkattack",
	 damage                = {
		default = 140,
	}
}

local BossOrkAxe = FancyMeleeWeapon:New {
	name                  = "BossOrkAxe",
	range                 = 200,
	reloadTime            = 2.5,
	areaOfEffect          = 300,
	-- targeting & accuracy
	accuracy              = 0.9,
	 -- soundStart            = [[SpearThrow]],
	soundStart             = "bossorkattack",
	 damage                = {
		default = 450,
	}
}


local Bite = FancyMeleeWeapon:New {
	name                  = "Bite",
	range                 = 80,
	reloadTime            = 0.5,
	-- targeting & accuracy
	accuracy              = 0.9,
	impactOnly            = true,
	 -- soundStart            = [[SpearThrow]],
	soundStart             = "smallorkattack",
	 damage                = {
		default = 9,
	}
}

local Firebomb = Weapon:New {
	name                  = "Firebomb",
	range                 = 350,
	reloadTime            = 15,
	-- targeting & accuracy
	-- general
	weaponType            = "Cannon", -- there's a default, but honestly, why?
	impactOnly            = true,
	noSelfDamage          = true,
	weaponVelocity        = 550,
	myGravity             = 1.2,
	tolerance             = 6000,
	highTrajectory        = 1,
	turret                = true,
	 -- collision & avoidance
	avoidFriendly         = false,
	avoidFeature          = false,
	 collideFriendly      = false,
	collideFeature        = false,
	
	-- If you uncomment this, then both orksmall and orkbig will lose the ability to attack.
	--customParams = {
	--	light_radius = 350,
	--	light_fade_time = 1,
	--	light_fade_offset = 10,
	--}
	
	 impulseFactor = 0,
	 -- targeting & accuracy
	accuracy              = 0.1,
	-- model                 = 'spear.dae',
	 -- soundStart            = [[SpearThrow]],
	soundStart             = "firebombfire",
	soundHit               = "firebombhit",
	 damage                = {
		default = 450,
	}
}


return {
	BigOrkAxe = BigOrkAxe,
	BossOrkAxe = BossOrkAxe,
	Bite = Bite,
	Firebomb = Firebomb,
}
