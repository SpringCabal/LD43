local moveBob = include("moveBob.lua")

local Hand_Right = piece("Hand_Right")
local Torso = piece("Torso")

function script.QueryWeapon()
	return Hand_Right
end

function script.AimWeapon()
	return true
end

function script.FireWeapon()
	moveBob.Attack()
end


function script.Create()
	moveBob.Init(Torso, 3)
end

function script.StartMoving()
	moveBob.StartMoving()
end

function script.StopMoving()
	moveBob.StopMoving()
end

function script.StartBuilding(heading, pitch)
	SetUnitValue(COB.INBUILDSTANCE, 1)
end