local Hand_Right = piece("Hand_Right")
local Torso = piece("Torso")
local isSword = false

local SIG_RESET = 32
local eyeRange = 420
local weaponRange = 70

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
		eyeRange = 460
		shared.InitSound("sounds/bravedeathyell.wav", 0.6)
	else
		eyeRange = 300
		shared.InitSound("sounds/panicdeathyell.wav", 0.6)
	end
	
	local ux, uy, uz = Spring.GetUnitPosition(unitID)
	Spring.GiveOrderToUnit(unitID, CMD.REPEAT, {1}, {})
	Spring.GiveOrderToUnit(unitID, CMD.FIGHT, {ux, uy, uz}, {})
	
	shared.Init(Torso, 3)
end

function script.StartMoving()
	shared.StartMoving()
end

function script.StopMoving()
	shared.StopMoving()
end

local function ResetMaxRange()
	Signal(SIG_RESET)
	SetSignalMask(SIG_RESET)
	
	Sleep(4500)
	Spring.SetUnitMaxRange(unitID, eyeRange)
	Spring.SetUnitWeaponState(unitID, 2, "range", eyeRange)
end

function script.BlockShot(num, targetID)
	if num == 2 then -- Eyes
		Spring.SetUnitMaxRange(unitID, weaponRange)
		Spring.SetUnitWeaponState(unitID, 2, "reloadFrame", Spring.GetGameFrame() + 30)
		Spring.SetUnitWeaponState(unitID, 2, "range", weaponRange)
		StartThread(ResetMaxRange)
		return true
	end
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