#version 330
// vim: ft=glsl

// Input vertex attributes
in vec3 vertexPosition;
in vec3 vertexNormal;
in vec4 vertexColor;

// Input uniform values
uniform mat4 mvp;

// Output vertex attributes (to fragment shader)
smooth out vec3 fragPosition;
out vec4 fragColor;
out vec3 fragNormal;

// NOTE: Add here your custom variables

void main()
{
    // Send vertex attributes to fragment shader
    fragPosition = vertexPosition;
    fragColor = vertexColor;
    fragNormal = vertexNormal;

    // Calculate final vertex position
    gl_Position = mvp*vec4(vertexPosition, 1.0);
}
