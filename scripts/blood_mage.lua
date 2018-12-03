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

local function DoCastAnimation()
	Turn(Hand_Right, x_axis, math.rad(200), math.rad(100))
	Turn(Hand_Left, x_axis, math.rad(200), math.rad(100))
	Turn(Hand_Right, z_axis, -math.rad(50), math.rad(100))
	Turn(Hand_Left, z_axis, math.rad(50), math.rad(100))

	Move(Torso, z_axis, 40, 100)
	Sleep(1000)
	Move(Torso, z_axis, 0, 500)

	Turn(Hand_Left, z_axis, math.rad(0), math.rad(200))
	Turn(Hand_Right, z_axis, math.rad(0), math.rad(200))
	Turn(Hand_Left, x_axis, math.rad(0), math.rad(200))
	Turn(Hand_Right, x_axis, math.rad(0), math.rad(200))
end

function script.CastAnimation()
	StartThread(DoCastAnimation)
end

function script.QueryWeapon()
	return Torso
end

function script.AimWeapon()
	return true
end

local shared = include("shared.lua")
function script.Create()
	shared.Init(Torso)
end

function script.StartMoving()
	shared.StartMoving()
end

function script.StopMoving()
	shared.StopMoving()
end

function script.BlockShot(num, targetID)
	shared.FaceTarget(targetID)
	return false
end

function script.FireWeapon()
	if GG.fireTx then
		local ux, _, uz = Spring.GetUnitPosition(unitID)
		local dx, dz = GG.fireTx - ux, GG.fireTz - uz
		shared.FaceDirection(dx, dz)
	end
	Spring.ClearUnitGoal(unitID)
	StartThread(AttackAnimation)
	return true
end