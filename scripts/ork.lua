local Hand_Right = piece("Hand_Right")
local Hand_Left = piece("Hand_Left")
local Torso = piece("Torso")
local unitDefName = "orksmall"
local tossRadius
local tossStr
local deunorkable

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

local function UpdatePosition()
	local x, _, z = Spring.GetUnitPosition(unitID)
	if x then
		Spring.SetGameRulesParam("boss_x", x)
		Spring.SetGameRulesParam("boss_z", z)
	end
	Sleep(2000)
end

function script.Create()
	local unitDefID = Spring.GetUnitDefID(unitID)
	unitDefName = UnitDefs[unitDefID].name
	Spring.SetUnitRulesParam(unitID, "unorkable", 1)
	if unitDefName == "orkboss" then
		shared.Init(Torso, 2, 2)
		tossRadius = 200
		tossStr = 10
		deunorkable = true
		StartThread(UpdatePosition)
		shared.InitSound("sounds/bossdeathyell.wav", 1)
		Spring.SetUnitMaxRange(unitID, 200)
	elseif unitDefName == "orkbig" then
		shared.Init(Torso, 2, 2)
		tossRadius = 120
		tossStr = 5
		shared.InitSound("sounds/bigorkdeathyell.wav", 1)
		Spring.SetUnitMaxRange(unitID, 120)
		if not Spring.MoveCtrl.GetTag(unitID) then
	else
		shared.Init(Torso, 3, 0.9)
		shared.InitSound("sounds/smallorkdeathyell.wav", 0.6)
		Spring.SetUnitMaxRange(unitID, 80)
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
			local targetUnorkable = Spring.GetUnitRulesParam(units[i], "unorkable")
			if units[i] ~= unitID and (deunorkable or (not targetUnorkable)) then
				local ux, uy, uz = Spring.GetUnitPosition(units[i])
				local dx, dz = ux - x, uz - z
				local dist = math.sqrt(dx*dx + dz*dz)
				local impulse = ((targetUnorkable and 0.5) or 1)*tossStr*(1 - dist/(3*tossRadius))
				Spring.AddUnitImpulse(units[i], impulse*dx/dist, impulse, impulse*dz/dist)
			end
		end
	end
	return false
end

function script.FireWeapon(num)
	if num == 1 then
		shared.AttackBob()
	end
	--StartThread(AttackAnimation) -- Broken
end