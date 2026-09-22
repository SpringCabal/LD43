-- feature_poof

return {
  ["fireball_ball"] = {
    explosionwave = {
      class              = [[CSimpleParticleSystem]],
      count              = 1,
      air                = true,
      ground             = true,
      water              = true,
      underwater         = true,
      unit               = true,
      nounit             = true,

      properties = {
        airdrag            = 0.87,
        alwaysvisible      = true,
        -- colormap           = [[1 0 0 0.15	0 0 0 0.0]],
        directional        = false,
        emitrot            = 0,
        -- emitrotspread      = 180,
        -- emitvector         = [[0, 0.1, 0]],
        gravity            = [[0, 0, 0]],
        numParticles       = 1,
        particleLife       = 5,
        particleLifeSpread = 2,
        particleSize       = 22,
        particleSizeSpread = 4,
        particleSpeed      = 0,
        particleSpeedSpread = 0.5,
        -- pos                = [[r20 r-20, -50, r20 r-20]],
        sizegrowth         = 1, -- same as groundflash circlegrowth
        sizemod            = 1,

        colormap = "1.0 0.25 0.02 1.0 1.0 0.75 0.08 0.95 0.2 0.02 0.0 0.0",
        -- texture            = [[fireball]],
        texture            = [[fireball_rot]],
      },
    },
  }
}

