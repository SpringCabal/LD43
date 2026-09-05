      uniform sampler2D tex0;
      uniform int useEqualityTest;
      uniform vec2 screenXY;

      void main(void)
      {
        vec2 texCoord = vec2( gl_FragCoord.x/screenXY.x , gl_FragCoord.y/screenXY.y );
        float depth  = texture2D(tex0, texCoord ).z;

        if (depth < gl_FragCoord.z) {
//          discard;
        }
        gl_FragColor = vec4(0.5, 0.5, 0.2, 0.4);
      }
