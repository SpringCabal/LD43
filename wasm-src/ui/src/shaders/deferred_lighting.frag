#define LIGHTRADIUS lightpos.w

uniform sampler2D modelnormals;
uniform sampler2D modeldepths;
uniform sampler2D mapnormals;
uniform sampler2D mapdepths;
uniform sampler2D modelExtra;

uniform vec3 eyePos;
uniform vec4 lightpos;
uniform vec4 lightcolor;
uniform mat4 viewProjectionInv;

float attenuate(float dist, float radius) {
    float raw = 0.7 - 0.3 * dist / radius - dist * dist / (radius * radius);
    float att = clamp(raw, 0.0, 1.0);
    return att * att;
}

void main(void)
{
    float mapDepth = texture2D(mapdepths, gl_TexCoord[0].st).x;
    float modelDepth = texture2D(modeldepths, gl_TexCoord[0].st).x;

#if (CLIP_CONTROL == 1)
    vec4 mapPosition = vec4(vec3(gl_TexCoord[0].st * 2.0 - 1.0, mapDepth), 1.0);
    vec4 modelPosition = vec4(vec3(gl_TexCoord[0].st * 2.0 - 1.0, modelDepth), 1.0);
#else
    vec4 mapPosition = vec4(vec3(gl_TexCoord[0].st, mapDepth) * 2.0 - 1.0, 1.0);
    vec4 modelPosition = vec4(vec3(gl_TexCoord[0].st, modelDepth) * 2.0 - 1.0, 1.0);
#endif
    vec4 mapNormal = texture2D(mapnormals, gl_TexCoord[0].st) * 2.0 - 1.0;
    vec4 modelNormal = texture2D(modelnormals, gl_TexCoord[0].st) * 2.0 - 1.0;
    vec4 modelExtraValue = texture2D(modelExtra, gl_TexCoord[0].st) * 2.0 - 1.0;

    float specularHighlight = 1.0;
    float modelLightingMultiplier = 1.0;
    if ((mapPosition.z - modelPosition.z) > 0.0 && modelExtraValue.a > 0.5) {
        mapNormal = modelNormal;
        mapPosition = modelPosition;
        modelLightingMultiplier = 1.5;
        specularHighlight += 2.0 * modelExtraValue.g;
    }

    mapPosition = viewProjectionInv * mapPosition;
    mapPosition.xyz /= mapPosition.w;

    vec3 lightDirection = normalize(lightpos.xyz - mapPosition.xyz);
    float distanceToLight = dot(lightpos.xyz - mapPosition.xyz, lightDirection);
    float cosPhi = max(0.0, dot(normalize(mapNormal.xyz), lightDirection));
    float attenuation = attenuate(distanceToLight, LIGHTRADIUS);
    vec3 viewDirection = normalize(eyePos - mapPosition.xyz);

    if (dot(mapNormal.xyz, lightDirection) > 0.02) {
        vec3 reflection = reflect(-lightDirection, mapNormal.xyz);
        float glossiness = dot(reflection, viewDirection);
        float highlight = pow(max(0.0, glossiness), 8.0);
        specularHighlight *= 0.5 * highlight;
    } else {
        specularHighlight = 0.0;
    }

    float lightAlpha = cosPhi * attenuation + attenuation * specularHighlight;
    lightAlpha = clamp(lightAlpha, 0.0, lightAlpha * ((mapPosition.y + 50.0) * 0.02));
    gl_FragColor = vec4(lightcolor.rgb * lightAlpha * modelLightingMultiplier, 1.0);
}
