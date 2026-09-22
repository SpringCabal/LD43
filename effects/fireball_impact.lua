return {
    ["fireball_impact"] = {
        usedefaultexplosions = false,

        groundflash = {
            circlealpha = 0.85,
            circlegrowth = 18,
            flashalpha = 1.4,
            flashsize = 95,
            ttl = 18,
            color = { [1] = 1.0, [2] = 0.22, [3] = 0.03 },
        },

        sphere = {
            air = true,
            ground = true,
            water = true,
            underwater = true,
            class = [[CSpherePartSpawner]],
            count = 1,
            properties = {
                alpha = 0.65,
                color = [[1,0.18,0.02]],
                expansionspeed = 18,
                ttl = 18,
            },
        },

        embers = {
            air = true,
            ground = true,
            class = [[CExpGenSpawner]],
            count = 8,
            properties = {
                delay = [[i1 x0.15d]],
                damage = [[d1]],
                explosionGenerator = [[custom:fireball_ember]],
            },
        },
    },

    ["fireball_ember"] = {
        particle = {
            air = true,
            ground = true,
            class = [[CSimpleParticleSystem]],
            count = 1,
            properties = {
                airdrag = 0.96,
                alwaysvisible = true,
                colormap = [[1.0 0.8 0.12 1.0 1.0 0.16 0.01 0.8 0.1 0.02 0.0 0.0]],
                directional = true,
                emitrot = 90,
                emitrotspread = 180,
                emitvector = [[0, 1, 0]],
                gravity = [[0, -0.08, 0]],
                numparticles = 1,
                particlelife = 20,
                particlelifespread = 8,
                particlesize = 7,
                particlesizespread = 4,
                particlespeed = 2.5,
                particlespeedspread = 3,
                pos = [[r-12 r12, 0, r-12 r12]],
                sizegrowth = -0.18,
                sizemod = 0.9,
                texture = [[fireball_rot]],
            },
        },
    },
}
