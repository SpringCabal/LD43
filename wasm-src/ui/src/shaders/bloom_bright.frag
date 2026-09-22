uniform sampler2D texture0;
uniform float illuminationThreshold;

void main(void) {
    vec2 texCoord = gl_TexCoord[0].st;
    vec3 color = texture2D(texture0, texCoord).rgb;
    float illumination = dot(color, vec3(0.2990, 0.5870, 0.1140));

    if (illumination > illuminationThreshold) {
        // Keep the blur in an 8-bit buffer while retaining the bright part of
        // the HDR image, exactly as the original Lua bloom pass does.
        color -= color * (illuminationThreshold / max(illumination, 0.00001));
        gl_FragColor = vec4(color, 1.0);
    } else {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    }
}
