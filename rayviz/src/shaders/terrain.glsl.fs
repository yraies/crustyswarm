#version 330
// vim: ft=glsl

// Input vertex attributes (from vertex shader)
in vec4 fragColor;
in vec3 fragPosition;
in vec3 fragNormal;
in vec3 viewDirection;

uniform bool drawHeightLines;

// Output fragment color
out vec4 finalColor;

void main() {
    //vec3 normal = normalize(fragNormal);
    //vec3 view = normalize(viewPos - fragPosition);
    //vec3 refl = reflect(-view, normal);

    float newOpacity = min(1.0, 0.7 / abs(dot(viewDirection, fragNormal)));
    float lightness = 0.6 + 0.4 * smoothstep(0,1,fragNormal.y);

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
    //val = val * lightness;

    //vec3 myColor = vec3(lightness,lightness,lightness);
    //vec3 myColor = fragTangent;
    vec3 myColor = vec3(newOpacity*val*lightness);

    // Calculate final fragment color
    finalColor = vec4(myColor,  1.0);
}
