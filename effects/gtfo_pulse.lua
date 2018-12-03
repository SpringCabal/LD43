return {
    ["gtfo_pulse"] = {
		usedefaultexplosions = false,
		
		groundflash = {
		  circlealpha        = 0.5,
		  circlegrowth       = 55,
		  flashalpha         = 1.2,
		  flashsize          = 68,
		  ttl                = 18,
		  color = {
			[1]  = 0.2,
			[2]  = 1,
			[3]  = 0,
		  },
		},
		sphere = {
		  air                = true,
		  class              = [[CSpherePartSpawner]],
		  count              = 1,
		  ground             = true,
		  water              = true,
		  underwater 		 = true,
		  properties = {
			alpha              = 0.5,
			color              = [[0.2,1,0]],
			expansionspeed     = 55,
			ttl                = 18,
		  },
		},
      },
}