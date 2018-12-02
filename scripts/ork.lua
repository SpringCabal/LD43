local Hand_Right = piece("Hand_Right")
local Hand_Left = piece("Hand_Left")
local Torso = piece("Torso")

local function AttackAnimation()
	Move(Hand_Right, y_axis, -30, 1000)
	Turn(Hand_Right, x_axis, -math.rad(20), math.rad(200))
	Turn(Hand_Right, z_axis, math.rad(30), math.rad(100))
	Sleep(100)
	Move(Hand_Right, y_axis, 0, 500)
	Turn(Hand_Right, x_axis, math.rad(0), math.rad(40))
	Turn(Hand_Right, z_axis, math.rad(0), math.rad(50))
end

function script.QueryWeapon()
	return Torso
end

function script.AimWeapon()
	return true
end

local moveBob = include("moveBob.lua")
function script.Create()
	moveBob.Init(Torso, 3)
end

function script.StartMoving()
	moveBob.StartMoving()
end

function script.StopMoving()
	moveBob.StopMoving()
end

function script.FireWeapon()
	moveBob.Attack()
	Spring.ClearUnitGoal(unitID)
	--StartThread(AttackAnimation) -- Broken
	return true
end