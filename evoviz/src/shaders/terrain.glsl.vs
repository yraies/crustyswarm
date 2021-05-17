#version 330
// vim: ft=glsl

in vec3 vertexPosition;
in vec3 vertexNormal;
in vec4 vertexColor;

uniform mat4 mvp;
uniform mat4 matModel;
uniform vec3 viewPos;

uniform bool drawHeightLines;

out vec3 fragPosition;
out vec4 fragColor;
out vec3 fragNormal;
out vec3 viewDirection;

void main()
{
    // Send vertex attributes to fragment shader
    fragPosition = vec3(matModel*vec4(vertexPosition, 1.0));
    fragColor = vertexColor;

    mat3 normalMatrix = transpose(inverse(mat3(matModel)));
    fragNormal = normalize(normalMatrix*vertexNormal);

    viewDirection = normalize(-(viewPos));


    // Calculate final vertex position
    gl_Position = mvp*vec4(vertexPosition, 1.0);
}

