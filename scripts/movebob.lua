local moveBob = {}
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

function moveBob.Attack()
	Move(bobPiece, y_axis, -20, 500)
	Turn(bobPiece, y_axis, -math.rad(20), math.rad(200))
	Sleep(100)
	Move(bobPiece, y_axis, 0, 250)
	Turn(bobPiece, y_axis, math.rad(0), math.rad(120))
end

function moveBob.Init(_bobPiece, _bobScale)
	bobPiece = _bobPiece
	bobScale = _bobScale or bobScale
end

function moveBob.StartMoving()
	StartThread(Bob)
end

function moveBob.StopMoving()
	Signal(SIG_MOVE)
	Move(bobPiece, y_axis, 0, 10)
end

return moveBob