local BloodMage = Raw:New {

	-- FIXME: The docs for buildPic are a lie.
	-- Explicitly specifying an empty string gives different results.
	-- Nvm, this seems to be a side effect of using the OOP class system
	-- That said, it shouldn't have any defaults.
	buildPic 			= "",
	-- Sensors
	sightDistance       = 800,
	mass                = 900,

	-- Commands
	canMove             = true,
	-- All these canX properties shouldn't be on by default:
	-- 	should they even be part of the engine? Lua it!
	canPatrol           = false,
	canGuard            = false,
	canRepeat           = false,
	fireState           = 0,

	customParams = {
	},
	-- Movement & Placement
	-- Wiki: (this section should be split into building and non-building parts)
	-- Maybe even needs to be split differently for ships/air/ground
	footprintX          = 2,
	footprintZ          = 2,
	upright             = true,
	minCollisionSpeed   = 1000000,
	pushResistant       = false,
	maxVelocity         = 22,
	turnInPlace         = false,
	turnInPlaceSpeedLimit = 0,
	-- maxVelocity's default value of "0" is odd, unless the default is a "building"?
	-- then again, acceleration is not 0 by default..
	acceleration        = 10,
	brakeRate           = 10,
	turnRate            = 1000 / 0.16,
	-- degrees per seconds = 0.16 * turnRate
	-- what a bizarre calculation, is this turnRate / (2 * pi)?
	-- this is probably just turnRate in radians and should be named such


	-- turnInPlace = true, (default)
	-- So it seems this default value is modeling a person rather than a car
	-- Why is the 'upright' also not reflecting that?
	-- Our default values should model *something*, otherwise they're pointless

	-- blocking & Spring.SetUnitBlocking
	-- we cannot modify all blocking properties here, and we need to rely on Lua

	-- Categories (probably don't need by default?)
	-- OH IT WAS NEEDED.
	category            = "INFANTRY",

	-- Collision Volumes
	-- This should be a good default for games that don't care about performance
	-- Let's hope it works...
	collisionVolumeType      = 'CylY',
	collisionVolumeScales    = '28 40 28',

	selectionVolumeOffsets = [[0 0 0]],
	selectionVolumeScales  = [[2 2 2]],
	selectionVolumeType    = [[ellipsoid]],

	-- General
	name                = "BloodMage",
	movementClass       = "Player",
	objectName          = "BloodMage.dae",
	script              = "blood_mage.lua",
	maxDamage           = 3200,

	iconType = "bloodmage",

	weapons = {
		{
			name = "Staff",
			onlyTargetCategory = [[INFANTRY HOUSE]],
		}
	},
}


return {
	BloodMage   = BloodMage
}