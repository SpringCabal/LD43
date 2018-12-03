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
        directional        = true,
        emitrot            = 0,
        -- emitrotspread      = 180,
        -- emitvector         = [[0, 0.1, 0]],
        gravity            = [[0, 0, 0]],
        numParticles       = 1,
        particleLife       = 2, -- same as groundflash ttl
        particleLifeSpread = 0,
        particleSize       = 20,
        particleSizeSpread = 1,
        particleSpeed      = 1,
        particleSpeedSpread = 10,
        -- pos                = [[r20 r-20, -50, r20 r-20]],
        sizegrowth         = 1, -- same as groundflash circlegrowth
        sizemod            = 1,
        -- texture            = [[fireball]],
        texture            = [[fireball_rot]],
      },
    },
  }
}

