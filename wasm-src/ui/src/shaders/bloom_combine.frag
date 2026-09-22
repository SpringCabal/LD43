uniform sampler2D texture0;
uniform sampler2D texture1;
uniform float fragMaxBrightness;

vec3 toneMapReinhard(vec3 color) {
    const float whitePoint = 1.0;
    float luminance = dot(color, vec3(0.2990, 0.5870, 0.1140));
    float mapped = (luminance * (1.0 + luminance / (whitePoint * whitePoint)))
        / (luminance + 1.0);
    return color * mapped / luminance;
}

void main(void) {
    vec2 texCoord = gl_TexCoord[0].st;
    vec4 hdr = texture2D(texture0, texCoord)
        + texture2D(texture1, texCoord) * fragMaxBrightness;

    const float whiteStart = 1.0;
    const float whiteScale = 10.0;
    float maximum = max(hdr.r, max(hdr.g, hdr.b));
    if (maximum > whiteStart) {
        hdr.rgb = mix(
            hdr.rgb,
            vec3(maximum),
            (maximum - whiteStart) / (((maximum - whiteStart) * whiteScale) + 1.0)
        );
    }

    // The original shader receives non-black pixels here, so luminance is
    // positive in normal use. Avoid a NaN if an empty framebuffer is sampled.
    if (dot(hdr.rgb, vec3(0.2990, 0.5870, 0.1140)) > 0.00001) {
        hdr.rgb = toneMapReinhard(hdr.rgb);
    } else {
        hdr.rgb = vec3(0.0);
    }
    gl_FragColor = vec4(hdr.rgb, 1.0);
}
