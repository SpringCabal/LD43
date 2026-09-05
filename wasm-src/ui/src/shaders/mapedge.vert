		// Application to vertex shader
		uniform float mirrorX;
		uniform float mirrorZ;
		uniform float lengthX;
	  uniform float lengthZ;
		uniform float left;
		uniform float up;
		uniform float brightness;

		varying vec4 vertex;
		varying vec4 color;

		void main()
		{
		gl_TexCoord[0]= gl_TextureMatrix[0]*gl_MultiTexCoord0;
		gl_Vertex.x = abs(mirrorX-gl_Vertex.x);
		gl_Vertex.z = abs(mirrorZ-gl_Vertex.z);

		float alpha = 1.0;

		float ff = 20000.0;
		if((mirrorZ && mirrorX))
	ff=ff/(pow(abs(gl_Vertex.z-up*mirrorZ)/150.0, 2.0)+pow(abs(gl_Vertex.x-left*mirrorX)/150.0, 2.0)+2.0);
		else if(mirrorX)
	ff=ff/(pow(abs(gl_Vertex.x-left*mirrorX)/150.0, 2.0)+2.0);
		else if(mirrorZ)
	ff=ff/(pow(abs(gl_Vertex.z-up*mirrorZ)/150.0, 2.0)+2.0);

		gl_Position  = gl_ModelViewProjectionMatrix*gl_Vertex;

		gl_FrontColor = vec4(brightness * gl_Color.rgb, alpha);

		color = gl_FrontColor;
		vertex = gl_Vertex;
		}
