--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
function gadget:GetInfo()
	return {
		name      = "Town Handler",
		desc      = "Spawns and handles towns.",
		author    = "GoogleFrog",
		date      = "2 December 2018",
		license   = "GNU GPL, v2 or later",
		layer     = 10,
		enabled   = true
	}
end

----------------------------------------------------------------------------
----------------------------------------------------------------------------
-- Config

local houseDefID = UnitDefNames["house"].id

local housePos = {
	{5488, 5296},
	{5200, 5488},
	{5200, 5680},
	{5200, 5984},
	{5952, 6176},
	{6144, 6080},
	{6144, 5888},
	{6144, 5488},
	{5952, 6368},
	{5008, 5984},
	{5968, 6736},
	{5968, 6928},
	{6256, 6752},
	{6256, 6944},
	{6160, 7248},
	{6352, 7248},
	{6752, 7072},
	{5968, 7120},
	{5296, 5200},
	{6752, 6880},
	{6560, 7024},
	{6544, 7216},
	{6448, 6752},
	{6432, 5888},
	{6432, 6080},
	{6224, 6368},
	{6416, 6368},
	{6608, 6368},
	{6624, 6176},
	{6624, 5984},
	{6336, 5488},
	{6528, 5488},
	{4960, 5104},
	{5008, 5376},
	{6944, 6800},
	{7056, 6592},
	{7056, 6400},
	{7056, 5808},
	{6960, 5616},
	{6960, 5424},
	{6960, 5232},
	{5968, 7312},
	{5776, 7312},
	{5392, 7312},
	{5200, 7312},
	{5008, 7312},
	{4816, 7312},
	{5376, 7504},
	{5792, 7504},
	{5248, 4912},
	{5056, 4912},
	{4864, 4912},
	{5632, 4480},
	{5440, 4480},
	{6016, 4560},
	{5920, 4368},
	{5840, 4176},
	{5456, 4224},
	{5648, 4224},
	{5456, 4032},
	{4912, 4656},
	{5024, 4240},
	{4752, 4240},
	{6832, 4672},
	{6640, 4704},
	{6448, 4720},
	{5536, 6176},
	{5344, 6176},
	{5536, 6832},
	{5536, 6640},
	{5536, 6448},
	{5344, 6832},
	{5152, 6832},
	{4960, 6832},
	{5152, 6176},
	{5904, 5296},
	{6096, 5296},
	{6384, 5216},
	{6576, 5184},
	{6768, 5152},
	{6256, 4784},
	{6224, 4592},
	{5728, 4672},
	{5760, 4864},
	{5952, 4848},
	{5024, 4048},
	{4832, 4048},
	{6768, 5488},
	{4720, 4656},
	{4592, 4848},
	{4672, 6656},
	{4672, 6464},
	{4672, 6272},
	{4672, 5888},
	{4592, 5040},
	{4672, 6848},
	{5264, 6560},
	{4960, 6640},
	{5200, 6368},
	{4944, 5792},
	{4624, 5696},
	{4704, 5504},
	{4512, 5424},
	{4624, 5232},
	{7056, 6000},
	{4720, 6080},
	{4864, 6320},
}

local villagerArea = {
	[1] = {
		x = 4900,
		z = 4950,
		width = 300,
		height = 400,
		units = {
			swordsman = 8,
			crossbowman = 2,
			coward = 2,
			peasant = 3,
		}
	},
	[2] = {
		x = 6370,
		z = 5000,
		width = 400,
		height = 400,
		units = {
			swordsman = 5,
			coward = 5,
			crossbowman = 2,
			peasant = 6,
		}
	},
	[3] = {
		x = 5600,
		z = 4500,
		width = 400,
		height = 400,
		units = {
			swordsman = 5,
			coward = 4,
			peasant = 3,
		}
	},
	[4] = {
		x = 5900,
		z = 6850,
		width = 500,
		height = 500,
		units = {
			swordsman = 6,
			coward = 9,
			peasant = 10,
		}
	},
	[5] = 	{
		x = 4930,
		z = 7070,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
		}
	},
	[6] = {
		x = 5580,
		z = 7330,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
		}
	},
	[7] = {
		x = 5400,
		z = 7050,
		width = 700,
		height = 300,
		units = {
			swordsman = 8,
			crossbowman = 6,
			peasant = 5,
		}
	},
	[8] = {
		x = 5740,
		z = 6320,
		width = 180,
		height = 300,
		units = {
			swordsman = 5,
			crossbowman = 2,
		}
	},
	[9] = {
		x = 6850,
		z = 6191,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
			crossbowman = 3,
		}
	},
	[10] = {
		x = 6700,
		z = 4900,
		width = 360,
		height = 180,
		units = {
			swordsman = 7,
			crossbowman = 3,
		}
	},
	[11] = {
		x = 6450,
		z = 5500,
		width = 300,
		height = 300,
		units = {
			swordsman = 5,
			crossbowman = 2,
			peasant = 3,
		}
	},
	[12] = {
		x = 4850,
		z = 4450,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
		}
	},
	[13] = {
		x = 5230,
		z = 4130,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
		}
	},
	[14] = {
		x = 5170,
		z = 4480,
		width = 240,
		height = 240,
		units = {
			swordsman = 4,
			crossbowman = 5,
		}
	},
	[15] = {
		x = 5600,
		z = 5100,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
			crossbowman = 2,
		}
	},
	[16] = {
		x = 4930,
		z = 5700,
		width = 300,
		height = 800,
		units = {
			swordsman = 6,
			crossbowman = 1,
			coward = 8,
			peasant = 5,
		}
	},
	[17] = {
		x = 5680,
		z = 5750,
		width = 1200,
		height = 1200,
		units = {
			swordsman = 25,
			crossbowman = 8,
			coward = 35,
			peasant = 24,
		}
	},
	[18] = {
		x = 5110,
		z = 6500,
		width = 700,
		height = 800,
		units = {
			swordsman = 12,
			crossbowman = 4,
			coward = 12,
			peasant = 8,
		}
	},
	[19] = {
		x = 6300,
		z = 6600,
		width = 900,
		height = 900,
		units = {
			swordsman = 7,
			crossbowman = 2,
			coward = 16,
			peasant = 12,
		}
	},
	[20] = {
		x = 5500,
		z = 5000,
		width = 1600,
		height = 800,
		units = {
			swordsman = 8,
			crossbowman = 3,
			coward = 20,
			peasant = 20,
		}
	},
	[21] = {
		x = 6500,
		z = 6000,
		width = 900,
		height = 1800,
		units = {
			swordsman = 15,
			crossbowman = 2,
			coward = 12,
			peasant = 18,
		}
	},
	[22] = {
		x = 6800,
		z = 5600,
		width = 300,
		height = 700,
		units = {
			swordsman = 2,
			crossbowman = 1,
			peasant = 1,
			coward = 2,
		}
	},
}

----------------------------------------------------------------------------
----------------------------------------------------------------------------
-- Spawning

local IterableMap = VFS.Include("LuaRules/Gadgets/Include/IterableMap.lua")
local villagers = IterableMap.New()

local function SpawnHouses()
	for i = 1, #housePos do
		local x = math.floor(housePos[i][1]/16)*16
		local z = math.floor(housePos[i][2]/16)*16
		local unitID = Spring.CreateUnit(houseDefID, x, Spring.GetGroundHeight(x, z), z, math.floor(math.random()*4), 0, false, false)
		Spring.SetUnitNoMinimap(unitID, true)
	end
end

local function SpawnVillager(area, unitDefID)
	local x = area.x + area.width*math.random() - area.width/2
	local z = area.z + area.height*math.random() - area.height/2
	local y = Spring.GetGroundHeight(x, z)
	local facing = math.floor(math.random()*4)
	local tries = 1
	while (tries < 4 and Spring.TestBuildOrder(unitDefID, x, y, z, facing) ~= 2) or 
		(tries >= 4 and tries < 8 and Spring.TestBuildOrder(unitDefID, x, y, z, facing) == 0) do
		x = area.x + area.width*math.random() - area.width/2
		z = area.z + area.height*math.random() - area.height/2
		y = Spring.GetGroundHeight(x, z)
		tries = tries + 1
	end
	local unitID = Spring.CreateUnit(unitDefID, x, y, z, math.floor(math.random()*4), 0, false, false)
	Spring.SetUnitRotation(unitID, 0, math.random()*math.pi, 0)
	-- Spring.SetUnitNoMinimap(unitID, true)
	--Spring.Utilities.UnitEcho(unitID, tries)
	villagers.Add(unitID, area)
	--Spring.MoveCtrl.SetGroundMoveTypeData(unitID, "maneuverLeash", 3) -- Does not appear to work
end

local function FillVillagerAreas()
	local totalVillagers = 0
	for i = 1, #villagerArea do
		--Spring.MarkerAddPoint(villagerArea[i].x, 0, villagerArea[i].z, i)
		for unitDefname, count in pairs(villagerArea[i].units) do
			Spring.Echo("unitDefname", unitDefname)
			local unitDefID = UnitDefNames[unitDefname].id
			for j = 1, count do
				SpawnVillager(villagerArea[i], unitDefID)
			end
			totalVillagers = totalVillagers + count
		end
	end
	Spring.Echo("totalVillagers", totalVillagers)
end

local function SpawnBloodMage(x, z)
	Spring.CreateUnit("bloodmage", x, Spring.GetGroundHeight(x, z), z, 0, 0, false, false)
end

----------------------------------------------------------------------------
----------------------------------------------------------------------------
-- Handling

local villagersKilled = 0

local function CheckIdle(unitID, area)
	if Spring.GetCommandQueue(unitID, 0) ~= 0 then
		return
	end
	local ux, uy, uz = Spring.GetUnitPosition(unitID)
	
	if math.abs(ux - area.x) < area.width and  math.abs(uz - area.z) < area.height then
		return
	end
	
	local x = area.x + area.width*math.random() - area.width/2
	local z = area.z + area.height*math.random() - area.height/2
	local dx = area.x + math.random()*area.width - area.width/2
	local dz = area.z + math.random()*area.height - area.height/2
	local dy = Spring.GetGroundHeight(dx, dz)
	Spring.GiveOrderToUnit(unitID, CMD.FIGHT, {dx, dy, dz}, {})
end

----------------------------------------------------------------------------
----------------------------------------------------------------------------
-- Callins

function gadget:GameFrame(frame)
	local count = math.ceil(villagers.GetIndexMax()/90)
	for i = 1, count do
		local unitID, area = villagers.Next()
		CheckIdle(unitID, area)
	end
end

function gadget:Initialize()
	local units = Spring.GetAllUnits()
	for i = 1, #units do
		Spring.SetUnitPosition(units[i], 100, 100)
		Spring.DestroyUnit(units[i], false, true)
	end
	
	SpawnHouses()
	SpawnBloodMage(5580, 7100)
	FillVillagerAreas()
end

function gadget:UnitDestroyed(unitID, unitDefID, teamID)
	Spring.SetUnitNoMinimap(unitID, true) -- Update minimap faster in the event of unit death.
	if villagers.Remove(unitID) then
		villagersKilled = villagersKilled + 1
		Spring.SetGameRulesParam("villagersKilled", villagersKilled)
	end
end

function gadget:Shutdown()
	local units = Spring.GetAllUnits()
	for i = 1, #units do
		Spring.SetUnitPosition(units[i], 100, 100)
		Spring.MoveCtrl.Enable(units[i])
		Spring.MoveCtrl.SetPosition(units[i], 100, 0, 100)
		Spring.UnitScript.SetDeathScriptFinished(units[i], 0)
		Spring.DestroyUnit(units[i], false, true)
	end
end