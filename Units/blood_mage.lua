local BloodMage = Humanoid:New {
	-- General
	name                = "BloodMage",
	movementClass       = "Defender",
	objectName 			= "BloodMage.dae",
	script              = "blood_mage.lua",
	maxDamage           = 1600,
	-- mass                = 50, -- does this even matter?

	maxVelocity         = 50,
	brakeRate           = 0.6,
	turnRate            = 760 / 0.16,

	weapons = {
		{
			name = "Axe",
		}
	},
}


return {
	BloodMage   = BloodMage
}