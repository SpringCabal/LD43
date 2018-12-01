function widget:GetInfo()
  return {
	name      = "Music for dummies",
	desc      = "",
	author    = "ashdnazg, gajop",
	date      = "yesterday",
	license   = "GPL-v2",
	layer     = 1001,
	enabled   = true,
  }
end

local VOLUME = 0.15
local BUFFER = 0.015

local playingTime = 0
local dtTime = 0
local trackTime
local startedPlaying = false
-- FIXME: add the music file path here
local musicFile

local function StartPlaying()
	playingTime = 0
	if not startedPlaying then
		Spring.PlaySoundStream(musicFile, VOLUME)
		_, trackTime = Spring.GetSoundStreamTime()
	end
	startedPlaying = true
end

function widget:Initialize()
	if not musicFile then
		widgetHandler:RemoveWidget()
		return
	end
	if Spring.GetGameFrame() > 0 then
		StartPlaying()
	end
end

function widget:GameStart()
	StartPlaying()
end

function widget:Update(dt)
	if startedPlaying then
		playingTime = playingTime + dt
		--playingTime = Spring.GetSoundStreamTime()
		if playingTime > trackTime - BUFFER then
		startedPlaying = false
		Spring.StopSoundStream()
		StartPlaying()
		end
	end
end

function widget:Shutdown()
	Spring.StopSoundStream()
end
