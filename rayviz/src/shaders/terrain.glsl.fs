#version 330
// vim: ft=glsl

// Input vertex attributes (from vertex shader)
in vec2 fragTexCoord;
in vec4 fragColor;
smooth in vec3 fragPosition;
in vec3 fragNormal;

uniform bool drawHeightLines;

// Output fragment color
out vec4 finalColor;




void main() {

    float val = 0.3;

    if (drawHeightLines) {
      val = fract(fragPosition.y / 10);

      if (val > 0.5) {
        val = 1 - val;
      }

      if (val < 0.01) {
        val = 0.075;
      } else if (val < 0.245) {
        val = 0.3;
      } else if (val < 0.255) {
        val = 0.25;
      } else if (val < 0.49) {
        val = 0.3;
      } else {
        val = 0.2;
      }
    }

    // Calculate final fragment color
    finalColor = vec4(val,val,val, 1.0);
}
