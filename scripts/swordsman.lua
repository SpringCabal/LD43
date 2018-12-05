local Hand_Right = piece("Hand_Right")
local Torso = piece("Torso")
local isSword = false

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
	return Hand_Right
end

function script.AimWeapon()
	return true
end

function script.StartBuilding(heading, pitch)
	SetUnitValue(COB.INBUILDSTANCE, 1)
end

local shared = include("shared.lua")
function script.Create()
	local unitDefID = Spring.GetUnitDefID(unitID)
	if UnitDefs[unitDefID].name == "swordsman" then
		isSword = true
		shared.InitSound("sounds/bravedeathyell.wav", 0.6)
	else
		shared.InitSound("sounds/panicdeathyell.wav", 0.6)
	end
	shared.Init(Torso, 3)
	if not Spring.MoveCtrl.GetTag(unitID) then
		Spring.MoveCtrl.SetGroundMoveTypeData(unitID, "maneuverLeash", 10)
	end
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
	shared.AttackBob()
	if isSword then
		StartThread(AttackAnimation)
	end
	return true
end