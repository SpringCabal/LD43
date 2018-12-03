return {
    ["migraine_pulse_spawner"] = {
        poof01 = {
        class              = [[CExpGenSpawner]],
        count              = [[1]],
        air                = true,
        ground             = true,
        water              = true,
        underwater         = true,
        unit               = true,
        nounit             = true,
        properties = {
                delay              = 5,
                damage             = [[d1]],
                explosionGenerator = [[custom:migraine_pulse]],
            },
        },
    },


    ["migraine_pulse"] = {
        explosionwave = {
          class              = [[CSimpleParticleSystem]],
          count              = 10,
          air                = true,
          ground             = true,
          water              = true,
          underwater         = true,
          unit               = true,
          nounit             = true,

          properties = {
            alwaysvisible      = true,
            -- colormap           = [[1 0 0 0.15	0 0 0 0.0]],
            directional        = false,
            -- emitrot            = 0,
            -- emitrotspread      = 180,
            numParticles       = 50,
            particleLife       = 0,
            particleLifeSpread = 33 * 0.5,
            particleSize       = 1,
            particleSizeSpread = 120,
            -- pos                = [[r20 r-20, -50, r20 r-20]],
            sizegrowth         = 10, -- same as groundflash circlegrowth
            sizemod            = 0.90,
            --texture            = [[fireball_rot]],
            texture            = [[migraine]],

            colormap = "0.5 0.1 1.0 0.0    1.0 0.1 0.5 1",
          },
        },
      },
}