local BigOrkAxe = MeleeWeapon:New {
	name                  = "BigOrkAxe",
	range                 = 150,
	reloadTime            = 3.0,
	-- targeting & accuracy
	accuracy              = 0.9,
	 -- soundStart            = [[SpearThrow]],
	soundHit              = [[Hit]],
	 damage                = {
		default = 300,
	}
}

local BossOrkAxe = MeleeWeapon:New {
	name                  = "BossOrkAxe",
	range                 = 200,
	reloadTime            = 3.0,
	-- targeting & accuracy
	accuracy              = 0.9,
	 -- soundStart            = [[SpearThrow]],
	soundHit              = [[Hit]],
	 damage                = {
		default = 600,
	}
}


local Bite = MeleeWeapon:New {
	name                  = "Bite",
	range                 = 80,
	reloadTime            = 0.5,
	-- targeting & accuracy
	accuracy              = 0.9,
	 -- soundStart            = [[SpearThrow]],
	soundHit              = [[Hit]],
	 damage                = {
		default = 20,
	}
}

local Firebomb = Weapon:New {
	--name                  = "Firebomb",
	--range                 = 250,
	--reloadTime            = 15,
	---- targeting & accuracy
	---- general
	--weaponType            = "Cannon", -- there's a default, but honestly, why?
	--impactOnly            = true,
	--noSelfDamage          = true,
	--weaponVelocity        = 500,
	--myGravity             = 1,
	--tolerance             = 6000,
	--turret                = true,
	-- -- collision & avoidance
	--avoidFriendly         = false,
	--avoidFeature          = false,
	-- collideFriendly      = false,
	--collideFeature        = false,
	--
	--customParams = {
	--	light_radius = 350,
	--	light_fade_time = 1,
	--	light_fade_offset = 10,
	--}
	--
	-- impulseFactor = 0,
	-- -- targeting & accuracy
	--accuracy              = 0.1,
	---- model                 = 'spear.dae',
	-- -- soundStart            = [[SpearThrow]],
	--soundHit              = [[Hit]],
	-- damage                = {
	--	default = 600,
	--}
}


return {
	BigOrkAxe = BigOrkAxe,
	BossOrkAxe = BossOrkAxe,
	Bite = Bite,
	Firebomb = Firebomb,
}
