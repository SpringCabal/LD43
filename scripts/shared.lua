local sharedFunc = {}
local SIG_MOVE = 1
local bobScale = 2

local bobPiece

local function Bob()
	Signal(SIG_MOVE)
	SetSignalMask(SIG_MOVE)
	
	while true do
		Move(bobPiece, z_axis, 3*bobScale, 20*bobScale)
		Sleep(166)
		Move(bobPiece, z_axis, 4*bobScale, 10*bobScale)
		Sleep(100)
		Move(bobPiece, z_axis, 3*bobScale, 10*bobScale)
		Sleep(100)
		Move(bobPiece, z_axis, 0*bobScale, 20*bobScale)
		Sleep(166)
		Move(bobPiece, z_axis, -1*bobScale, 10*bobScale)
		Sleep(100)
		Move(bobPiece, z_axis, 0*bobScale, 10*bobScale)
		Sleep(100)
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
end

function sharedFunc.StartMoving()
	StartThread(Bob)
end

function sharedFunc.StopMoving()
	Signal(SIG_MOVE)
	Move(bobPiece, y_axis, 0, 10)
end

function sharedFunc.FaceTarget(targetID)
	if not targetID then
		return
	end
	local ux, uy, uz = Spring.GetUnitPosition(unitID)
	local tx, ty, tz = Spring.GetUnitPosition(targetID)
	local angle = Spring.Utilities.Vector.Angle(tx - ux, tz - uz)
	Spring.MoveCtrl.SetHeading(unitID, angle)
end

return sharedFunc
