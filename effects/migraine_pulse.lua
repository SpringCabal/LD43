return {
    ["migraine_pulse_spawner"] = {
        poof01 = {
        class              = [[CExpGenSpawner]],
        count              = [[18]],
        air                = true,
        ground             = true,
        water              = true,
        underwater         = true,
        unit               = true,
        nounit             = true,
        properties = {
				delay              = [[0 i20]],
                damage             = [[d1]],
                explosionGenerator = [[custom:migraine_pulse]],
            },
        },
    },


    ["migraine_pulse"] = {
		usedefaultexplosions = false,
		
		groundflash = {
		  circlealpha        = 1,
		  circlegrowth       = 12,
		  flashalpha         = 2.15,
		  flashsize          = 68,
		  ttl                = 22,
		  color = {
			[1]  = 1,
			[2]  = 0.89999997615814,
			[3]  = 0.60000002384186,
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
			color              = [[1,0,1]],
			expansionspeed     = 12,
			ttl                = 22,
		  },
		},
      },
}