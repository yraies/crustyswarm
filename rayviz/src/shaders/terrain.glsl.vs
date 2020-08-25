#version 330
// vim: ft=glsl

// Input vertex attributes
in vec3 vertexPosition;
in vec3 vertexNormal;
in vec2 vertexTexCoord;
in vec4 vertexColor;
in vec4 vertexTangent;

// Input uniform values
uniform mat4 mvp;
uniform mat4 matModel;
uniform vec3 viewPos;

// Output vertex attributes (to fragment shader)
out vec3 fragPosition;
out vec4 fragColor;
out vec3 fragNormal;

// NOTE: Add here your custom variables

void main()
{

    // Calculate fragment normal based on normal transformations
    mat3 normalMatrix = transpose(inverse(mat3(matModel)));

    // Calculate fragment position based on model transformations
    fragPosition = vec3(matModel*vec4(vertexPosition, 1.0f));

    // Send vertex attributes to fragment shader
    fragColor = vertexColor;
    fragNormal = normalize(normalMatrix*vertexNormal);

    // Calculate final vertex position
    gl_Position = mvp*vec4(vertexPosition, 1.0);
}
