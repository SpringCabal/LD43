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

return {
	BigOrkAxe = BigOrkAxe,
	BossOrkAxe = BossOrkAxe,
	Bite = Bite
}
