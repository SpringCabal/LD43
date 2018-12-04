local Sounds = {
  SoundItems = {
	IncomingChat = {
	  --- always play on the front speaker(s)
	  file = "sounds/beep4.wav",
	  in3d = "false",
	},
	MultiSelect = {
	  --- always play on the front speaker(s)
	  file = "sounds/button9.wav",
	  in3d = "false",
	},
	MapPoint = {
	  --- respect where the point was set, but don't attenuate in distance
	  --- also, when moving the camera, don't pitch it
	  file = "sounds/beep6.wav",
	  rolloff = 0,
	  dopplerscale = 0,
	},
	digitout = {
	  --- some things you can do with this file
	  --- can be either ogg or wav
	  file = "sounds/digitout.wav",
	  in3d = "true",
	},
  },
}

local defaultOpts = {
	pitchmod = 0.02,
	gainmod = 0,
}

local VFSUtils = VFS.Include('gamedata/VFSUtils.lua')

local function AutoAdd(subDir, generalOpts)
	generalOpts = generalOpts or {}
	local opts
	local dirList = RecursiveFileSearch("sounds" .. subDir)
	--local dirList = RecursiveFileSearch("sounds/")
	--Spring.Echo("Adding sounds for " .. subDir)
	for _, fullPath in ipairs(dirList) do
		local path, key, ext = fullPath:match("sounds/(.*/(.*)%.(.*))")
		local pathPart = fullPath:match("(.*)[.]")
		pathPart = pathPart:sub(8, -1)	-- truncates extension fullstop and "sounds/" part of path
		--Spring.Echo(pathPart)
		if path ~= nil and (not ignoredExtensions[ext]) then
			if optionOverrides[pathPart] then
				opts = optionOverrides[pathPart]
				--Spring.Echo("optionOverrides for " .. pathPart)
			else
				opts = generalOpts
			end
			--Spring.Echo(path,key,ext, pathPart)
			Sounds.SoundItems[pathPart] = {
				file = tostring('sounds/'..path), 
				rolloff = opts.rollOff, 
				dopplerscale = opts.dopplerscale, 
				maxdist = opts.maxdist, 
				maxconcurrent = opts.maxconcurrent, 
				priority = opts.priority, 
				in3d = opts.in3d,
				gain = opts.gain, 
				gainmod = opts.gainmod, 
				pitch = opts.pitch, 
				pitchmod = opts.pitchmod
			}
			--Spring.Echo(Sounds.SoundItems[key].file)
		end
	end
end

-- add sounds
AutoAdd("", defaultOpts)

return Sounds