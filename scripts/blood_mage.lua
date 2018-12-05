local Hand_Right = piece("Hand_Right")
local Hand_Left = piece("Hand_Left")
local Torso = piece("Torso")

local shared = include("shared.lua")

local function AttackAnimation()
	Move(Hand_Right, y_axis, -35, 500)
	Turn(Hand_Right, x_axis, -math.rad(20), math.rad(240))
	Turn(Hand_Right, z_axis, math.rad(30), math.rad(120))
	
	Turn(Torso, x_axis, -math.rad(5 + math.random()*2), math.rad(120))
	Move(Torso, y_axis, -8 - math.random()*4, 45 + math.random()*10)
	Sleep(100)
	Move(Hand_Right, y_axis, 0, 200)
	Turn(Hand_Right, x_axis, math.rad(0), math.rad(70))
	Turn(Hand_Right, z_axis, math.rad(0), math.rad(80))
	
	Turn(Torso, x_axis, math.rad(0), math.rad(100))
	Move(Torso, y_axis, 0, 30)
end

local function SmallCast(castFunc)
	shared.StopMoving()
	
	Move(Hand_Right, x_axis, -50, 200)
	Move(Hand_Right, z_axis, 40, 150)
	
	Move(Hand_Left, x_axis, 50, 200)
	Move(Hand_Left, z_axis, 40, 150)
	
	Turn(Hand_Right, z_axis, math.rad(25), math.rad(160))
	Turn(Hand_Right, x_axis, -math.rad(82), math.rad(260))
	
	Turn(Hand_Left, z_axis, math.rad(50), math.rad(160))

	Sleep(300)
	if castFunc then
		castFunc()
	end
	Sleep(100)

	Move(Hand_Right, x_axis, 0, 100)
	Move(Hand_Right, z_axis, 0, 80)
	
	Move(Hand_Left, x_axis, 0, 100)
	Move(Hand_Left, z_axis, 0, 80)
	
	
	Turn(Hand_Right, z_axis, math.rad(0), math.rad(120))
	Turn(Hand_Right, x_axis, math.rad(0), math.rad(200))
	
	Turn(Hand_Left, z_axis, math.rad(0), math.rad(120))
end

local function MediumCast(castFunc, tx, tz)
	shared.StopMoving()
	if tx then
		local ux, _, uz = Spring.GetUnitPosition(unitID)
		local dx, dz = tx - ux, tz - uz
		shared.FaceDirection(dx, dz)
	end
	
	Move(Hand_Right, x_axis, -50, 200)
	Move(Hand_Right, z_axis, 40, 150)
	
	Move(Hand_Left, x_axis, 50, 200)
	Move(Hand_Left, z_axis, 40, 150)
	
	Turn(Hand_Right, z_axis, math.rad(25), math.rad(160))
	Turn(Hand_Right, x_axis, -math.rad(82), math.rad(260))
	
	Turn(Hand_Left, z_axis, math.rad(50), math.rad(160))

	Move(Torso, z_axis, 25, 100)
	Sleep(300)
	Move(Torso, z_axis, 30, 30)
	if castFunc then
		castFunc()
	end
	Sleep(60)
	Move(Torso, z_axis, 0, 20)
	Sleep(60)
	Move(Torso, z_axis, 0, 100)

	Move(Hand_Right, x_axis, 0, 100)
	Move(Hand_Right, z_axis, 0, 80)
	
	Move(Hand_Left, x_axis, 0, 100)
	Move(Hand_Left, z_axis, 0, 80)
	
	
	Turn(Hand_Right, z_axis, math.rad(0), math.rad(120))
	Turn(Hand_Right, x_axis, math.rad(0), math.rad(200))
	
	Turn(Hand_Left, z_axis, math.rad(0), math.rad(120))
end

local function LongCast(castFunc, tx, tz)
	shared.StopMoving()
	if tx then
		local ux, _, uz = Spring.GetUnitPosition(unitID)
		local dx, dz = tx - ux, tz - uz
		shared.FaceDirection(dx, dz)
	end
	
	Move(Hand_Right, x_axis, -50, 130)
	Move(Hand_Right, z_axis, 40, 110)
	
	Move(Hand_Left, x_axis, 50, 130)
	Move(Hand_Left, z_axis, 40, 110)
	
	Turn(Hand_Right, z_axis, math.rad(25), math.rad(120))
	Turn(Hand_Right, x_axis, -math.rad(82), math.rad(200))
	
	Turn(Hand_Left, z_axis, math.rad(50), math.rad(120))

	Move(Torso, z_axis, 40, 120)
	Sleep(450)
	if castFunc then
		castFunc()
	end
	Sleep(100)
	Move(Torso, z_axis, 0, 80)

	Move(Hand_Right, x_axis, 0, 100)
	Move(Hand_Right, z_axis, 0, 80)
	
	Move(Hand_Left, x_axis, 0, 100)
	Move(Hand_Left, z_axis, 0, 80)
	
	
	Turn(Hand_Right, z_axis, math.rad(0), math.rad(120))
	Turn(Hand_Right, x_axis, math.rad(0), math.rad(200))
	
	Turn(Hand_Left, z_axis, math.rad(0), math.rad(120))
end

local castAnimationSize = {SmallCast, MediumCast, LongCast} 

function script.CastAnimation(castFunc, level, tx, tz)
	StartThread(castAnimationSize[level], castFunc, tx, tz)
end

function script.QueryWeapon()
	return Hand_Right
end

function script.AimWeapon()
	return true
end

function script.Create()
	Spring.SetUnitRulesParam(unitID, "unorkable", 1)
	shared.Init(Torso)
	shared.InitSound("sounds/death_player.wav", 1)
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

function script.BlockShot()
	if not GG.fireTx then
		return true
	end
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