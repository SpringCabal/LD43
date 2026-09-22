uniform sampler2D texture0;
uniform float inverseRX;
uniform float fragKernelRadius;

void main(void) {
    vec2 texCoord = gl_TexCoord[0].st;
    float sigma = fragKernelRadius / 2.0;
    float weight = 1.0 / (2.50663 * sigma);
    float totalWeight = weight;
    vec4 sum = texture2D(texture0, texCoord) * weight;

    for (float radius = 1.5; radius < fragKernelRadius; radius += 2.0) {
        weight = exp(-((radius * radius) / (2.0 * sigma * sigma)))
            / (2.50663 * sigma);
        sum += texture2D(texture0, texCoord - vec2(radius * inverseRX, 0.0)) * weight;
        sum += texture2D(texture0, texCoord + vec2(radius * inverseRX, 0.0)) * weight;
        totalWeight += 2.0 * weight;
    }

    gl_FragColor = vec4(sum.rgb / totalWeight, 1.0);
}
