local Hand_Right = piece("Hand_Right")
local Hand_Left = piece("Hand_Left")
local Torso = piece("Torso")
local unitDefName = "orksmall"
local tossRadius
local tossStr

local shared = include("shared.lua")

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

function script.Create()
	local unitDefID = Spring.GetUnitDefID(unitID)
	unitDefName = UnitDefs[unitDefID].name
	if unitDefName == "orkboss" then
		shared.Init(Torso, 8, 10)
		tossRadius = 200
		tossStr = 12
		Spring.SetUnitRulesParam(unitID, "unorkable", 1)
	elseif unitDefName == "orkbig" then
		shared.Init(Torso, 5, 5)
		tossRadius = 120
		tossStr = 5
		Spring.SetUnitRulesParam(unitID, "unorkable", 1)
	else
		shared.Init(Torso, 3)
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
	if targetID and tossRadius then
		local tx, _, tz = Spring.GetUnitPosition(targetID)
		local x, _, z = Spring.GetUnitPosition(unitID)
		local units = Spring.GetUnitsInCylinder(tx, tz, tossRadius)
		for i = 1, #units do
			if not Spring.GetUnitRulesParam(units[i], "unorkable") then
				local ux, uy, uz = Spring.GetUnitPosition(units[i])
				local dx, dz = ux - x, uz - z
				local dist = math.sqrt(dx*dx + dz*dz)
				local impulse = tossStr*(1 - dist/(3*tossRadius))
				Spring.AddUnitImpulse(units[i], impulse*dx/dist, impulse, impulse*dz/dist)
			end
		end
	end
	return false
end

function script.FireWeapon(num)
	if num == 1 then
		shared.AttackBob()
		Spring.ClearUnitGoal(unitID)
	end
	--StartThread(AttackAnimation) -- Broken
end