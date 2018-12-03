local sharedFunc = {}
local SIG_MOVE = 1
local SIG_BUFF = 2
local SIG_STUN = 4

local bobScale = 2
local timeScale = 1
local staticBobScale = 2

local bobPiece

local function Bob()
	Signal(SIG_MOVE)
	SetSignalMask(SIG_MOVE)

	while true do
		Move(bobPiece, z_axis, 3*bobScale, 20*bobScale)
		Sleep(100*timeScale)
		Move(bobPiece, z_axis, 4*bobScale, 10*bobScale)
		Sleep(66*timeScale)
		Move(bobPiece, z_axis, 3*bobScale, 10*bobScale)
		Sleep(66*timeScale)
		Move(bobPiece, z_axis, 0*bobScale, 20*bobScale)
		Sleep(100*timeScale)
		Move(bobPiece, z_axis, -1*bobScale, 10*bobScale)
		Sleep(66*timeScale)
		Move(bobPiece, z_axis, 0*bobScale, 10*bobScale)
		Sleep(66*timeScale)
	end
end

function sharedFunc.AttackBob()
	Move(bobPiece, y_axis, -20, 500)
	Turn(bobPiece, y_axis, -math.rad(20), math.rad(200))
	Sleep(100)
	Move(bobPiece, y_axis, 0, 250)
	Turn(bobPiece, y_axis, math.rad(0), math.rad(120))
end

function sharedFunc.Init(_bobPiece, _bobScale)
	bobPiece = _bobPiece
	bobScale = _bobScale or bobScale
	staticBobScale = bobScale
end

function sharedFunc.StartMoving()
	StartThread(Bob)
end

function sharedFunc.StopMoving()
	Signal(SIG_MOVE)
	Move(bobPiece, y_axis, 0, 10)
end

function sharedFunc.FaceDirection(dx, dz)
	local angle = Spring.Utilities.Vector.Angle(dx, dz) - math.pi/2
	Spring.SetUnitRotation(unitID, 0, angle, 0)
end

function sharedFunc.FaceTarget(targetID)
	if not targetID then
		return
	end
	local ux, uy, uz = Spring.GetUnitPosition(unitID)
	local tx, ty, tz = Spring.GetUnitPosition(targetID)
	sharedFunc.FaceDirection(tx - ux, tz - uz)
end

local function BuffThread(duration)
	Signal(SIG_BUFF)
	SetSignalMask(SIG_BUFF)
	
	bobScale = staticBobScale*2
	timeScale = timeScale/2
	Sleep(duration*1000/30)
	
	bobScale = staticBobScale
	timeScale = 1
end

local function StunThread(duration)
	Signal(SIG_STUN)
	SetSignalMask(SIG_STUN)
	
	local dir = math.floor(math.random()*2)*2 - 1
	Spin(bobPiece, z_axis, dir*math.rad(250 + math.random()*100), math.rad(400))
	Turn(bobPiece, y_axis, math.rad(90), math.rad(90))
	bobScale = staticBobScale*0.1
	
	Sleep(duration*1000/30)
	
	Spin(bobPiece, z_axis, math.rad(0), math.rad(400))
	Turn(bobPiece, y_axis, math.rad(0), math.rad(90))
	bobScale = staticBobScale*2
end

function script.Buff(duration)
	StartThread(BuffThread, duration)
end

function script.Stun(duration)
	StartThread(StunThread, duration)
end

return sharedFunc
