--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
   return {
	  name      = "Game modes",
	  desc      = "Loads game modes from modoptions and sets the appropriate gamerules",
	  author    = "gajop",
	  date      = "15.04.2016.",
	  license   = "Public Domain",
	  layer     = 0,
	  enabled   = true
   }
end

local modOptions

function LoadGameMode(modeName)
	local modeValue = Spring.GetGameRulesParam(modeName)
	if modeValue == nil then
		modeValue = modOptions[modeName] or 0
		Spring.SetGameRulesParam(modeName, modeValue)
	end
	return modeValue
end

function SafeSetGameMode(gameMode)
	-- three valid game modes:
	-- develop -> camera isn't locked and input isn't grabbed, units aren't spawned initially
	-- test    -> camera is locked and input is grabbed, units are spawned, BUT it's possible to use the console and switch back to develop mode
	-- play    -> all the same as test except it's not possible to switch to develop
	if gameMode ~= "develop" and gameMode ~= "test" and gameMode ~= "play" then
		gameMode = "develop"
	end
	if Spring.GetGameRulesParam("gameMode") ~= gameMode then
		Spring.SetGameRulesParam("gameMode", gameMode)
	end
end

local function explode(div,str)
	if (div=='') then return 
		false 
	end
	local pos,arr = 0,{}
	-- for each divider found
	for st,sp in function() return string.find(str,div,pos,true) end do
		table.insert(arr,string.sub(str,pos,st-1)) -- Attach chars left of current divider
		pos = sp + 1 -- Jump past current divider
	end
	table.insert(arr,string.sub(str,pos)) -- Attach chars right of last divider
	return arr
end

function gadget:RecvLuaMsg(msg)
	local msg_table = explode('|', msg)
	if msg_table[1] == "setGameMode" then
		Spring.SetGameRulesParam("gameMode", msg_table[2])
		Spring.Echo("gameMode: ", msg_table[2])
	end
end

function gadget:Initialize()
	if Spring.GetGameRulesParam("gameMode") == nil then
		modOptions = Spring.GetModOptions()
		local gameMode = modOptions.gameMode
		SafeSetGameMode(gameMode)
	end
end

function gadget:GameFrame()
	local gameMode = Spring.GetGameRulesParam("gameMode")
	SafeSetGameMode(gameMode)
end
