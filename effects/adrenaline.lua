return {
    ["adrenaline_sparkles"] = {
        add_spawner = {
            class              = [[CExpGenSpawner]],
            count              = [[3]],
            air                = true,
            ground             = true,
            water              = true,
            underwater         = true,
            unit               = true,
            nounit             = true,
            properties = {
                    delay              = 0,
                    damage             = [[d1]],
                    explosionGenerator = [[custom:adrenaline_sparkles_efx]],
                },
            },
    },

    ["adrenaline_sparkles_efx"] = {
        explosionwave = {
            air                = true,
            class              = [[CSimpleParticleSystem]],
            count              = 1,
            ground             = true,
            water              = true,
            underwater         = true,

            properties = {
            airdrag            = 0.87,
            alwaysvisible      = true,
            colormap           = [[1 1 0 0.15 0 0 0 0.01]],
            directional        = false,
            emitrot            = 0,
            emitrotspread      = 180,
            emitvector         = [[0, 0, 1]],
            gravity            = [[0, -0.2, 0]],
            numparticles       = 50,
            particlelife       = 70, -- same as groundflash ttl
            particlelifespread = 0,
            particleSize       = 0.1,
            particleSizeSpread = 1,
            particleSpeed      = 2,
            particleSpeedSpread = 5,
            pos = {0, 10, 0},
            -- pos                = [[r20 r-20, -50, r20 r-20]],
            sizegrowth         = 0.1, -- same as groundflash circlegrowth
            sizemod            = 1.0,
            -- texture            = [[fireball]],
            texture            = [[migraine]],
            },
        },
    },
}