-- $Id: gfx_lups_units_on_fire.lua 3171 2008-11-06 09:06:29Z det $

function widget:GetInfo()
  return {
    name      = "Units on Fire",
    desc      = "Graphical effect for burning units",
    author    = "jK/quantum",
    date      = "Sep, 2008",
    license   = "GNU GPL, v2 or later",
    layer     = 10,
    enabled   = true  --  loaded by default?
  }
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local spGetUnitRulesParam = Spring.GetUnitRulesParam
local spGetAllUnits = Spring.GetAllUnits

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local flameFX = VFS.Include("LuaUI/Configs/ttld_fire.lua")

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

-- Match the Rust widget's isolated stream; every integer remains below 2^24.
local randomState = 1
local function sample()
    randomState = (randomState*251+1)%65521
    return randomState/65521
end
flameFX.random = sample
flameFX.partpos = "0,0,0"

local houses, burning = {}, {}
local function find(list,id)
    local low,high=1,#list
    while low<=high do
        local at=math.floor((low+high)/2)
        if list[at]==id then return at,true
        elseif list[at]<id then low=at+1 else high=at-1 end
    end
    return low,false
end
local function insert(list,id)
    local at,found=find(list,id)
    if not found then table.insert(list,at,id) end
end
local function remove(list,id)
    local at,found=find(list,id)
    if found then table.remove(list,at) end
end
local houseDefID = UnitDefNames.house.id
local AddParticles
function widget:UnitCreated(id,defID)
    if defID==houseDefID then
        local _,lit=find(burning,id)
        if not lit then insert(houses,id) end
    end
end
function widget:UnitDestroyed(id)
    remove(houses,id); remove(burning,id)
end
function widget:Initialize()
    if not WG.Lups then widgetHandler:RemoveWidget(self); return end
    AddParticles=WG.Lups.AddParticles
    for _,id in ipairs(Spring.GetAllUnits()) do self:UnitCreated(id,Spring.GetUnitDefID(id)) end
end
local lastPoll=0
function widget:Update()
    local frame=Spring.GetGameFrame()
    local poll=frame-frame%6
    if poll<=lastPoll then return end
    lastPoll=poll
    local at=1
    while at<=#houses do
        local id=houses[at]
        if Spring.GetUnitRulesParam(id,"on_fire")==1 then
            table.remove(houses,at); insert(burning,id)
        else at=at+1 end
    end
    if #burning==0 then return end
    local wx,wy,wz=Spring.GetWind()
    flameFX.force[1],flameFX.force[2],flameFX.force[3]=wx*0.09,wy*0.09+3,wz*0.09
    for _,id in ipairs(burning) do
        if math.floor(sample()*400)+1 < 400-#burning then
            local x,y,z=Spring.GetUnitPosition(id)
            local radius=Spring.GetUnitRadius(id)
            if x and radius then
                flameFX.pos[1],flameFX.pos[2],flameFX.pos[3]=x,y,z
                flameFX.size=radius*0.35
                flameFX.startRadius=radius*0.6
                AddParticles('SimpleParticles2',flameFX)
            end
        end
    end
end
