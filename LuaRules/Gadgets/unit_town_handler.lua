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
	{
		x = 4930,
		z = 7070,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
		}
	},
	{
		x = 5580,
		z = 7330,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
		}
	},
	{
		x = 5400,
		z = 7070,
		width = 500,
		height = 180,
		units = {
			crossbowman = 6,
		}
	},
	{
		x = 5740,
		z = 6300,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
			crossbowman = 2,
		}
	},
	{
		x = 6850,
		z = 6191,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
			crossbowman = 2,
		}
	},
	{
		x = 6600,
		z = 4900,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
			crossbowman = 2,
		}
	},
	{
		x = 6250,
		z = 5700,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
			crossbowman = 2,
		}
	},
	{
		x = 4850,
		z = 4450,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
		}
	},
	{
		x = 5230,
		z = 4130,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
		}
	},
	{
		x = 5170,
		z = 4480,
		width = 240,
		height = 240,
		units = {
			crossbowman = 5,
		}
	},
	{
		x = 5600,
		z = 5100,
		width = 180,
		height = 180,
		units = {
			swordsman = 5,
			crossbowman = 2,
		}
	},
	{
		x = 4930,
		z = 5700,
		width = 180,
		height = 400,
		units = {
			swordsman = 3,
			crossbowman = 1,
		}
	},
	{
		x = 5680,
		z = 5750,
		width = 900,
		height = 900,
		units = {
			swordsman = 3,
			crossbowman = 1,
			peasant = 60,
		}
	},
}

----------------------------------------------------------------------------
----------------------------------------------------------------------------

local IterableMap = VFS.Include("LuaRules/Gadgets/Include/IterableMap.lua")
local villagers = IterableMap.New()

local function SpawnHouses()
	for i = 1, #housePos do
		local x = math.floor(housePos[i][1]/16)*16
		local z = math.floor(housePos[i][2]/16)*16
		Spring.CreateUnit(houseDefID, x, Spring.GetGroundHeight(x, z), z, math.floor(math.random()*4), 0, false, false)
	end
end

local function SpawnVillager(area, unitDefID)
	local x = area.x + area.width*math.random() - area.width/2
	local z = area.z + area.height*math.random() - area.height/2
	local y = Spring.GetGroundHeight(x, z)
	local facing = math.floor(math.random()*4)
	local tries = 1
	while tries < 8 and Spring.TestBuildOrder(unitDefID, x, y, z, facing) ~= 2 do
		x = area.x + area.width*math.random() - area.width/2
		z = area.z + area.height*math.random() - area.height/2
		y = Spring.GetGroundHeight(x, z)
		tries = tries + 1
	end
	local unitID = Spring.CreateUnit(unitDefID, x, y, z, math.floor(math.random()*4), 0, false, false)
	--Spring.Utilities.UnitEcho(unitID, tries)
	villagers.Add(unitID, area)
end

local function FillVillagerAreas()
	for i = 1, #villagerArea do
		for unitDefname, count in pairs(villagerArea[i].units) do
			local unitDefID = UnitDefNames[unitDefname].id
			for j = 1, count do
				SpawnVillager(villagerArea[i], unitDefID)
			end
		end
	end
end

local function SpawnBloodMage(x, z)
	Spring.CreateUnit("bloodmage", x, Spring.GetGroundHeight(x, z), z, 0, 0, false, false)
end


----------------------------------------------------------------------------
----------------------------------------------------------------------------

function gadget:Initialize()
	local units = Spring.GetAllUnits()
	for i = 1, #units do
		Spring.DestroyUnit(units[i], false, true)
	end
	
	SpawnHouses()
	SpawnBloodMage(5700, 5740)
	FillVillagerAreas()
end
