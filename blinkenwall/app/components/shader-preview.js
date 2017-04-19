import Ember from 'ember';

const VERTICES = new Float32Array([
  -1, -1, 0, 0,
  -1,  1, 0, 1,
   1, -1, 1, 0,
   1,  1, 1, 1,
]);
const NUM_VERTICES = VERTICES.length / 4;

const VERTEX_SHADER = `#version 100

attribute vec2 position;
attribute vec2 texcoords;

varying vec2 vTexCoords;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    vTexCoords = texcoords;
}`;

const FRAGMENT_SHADER_PREAMBLE = `#version 100

precision mediump float;
varying vec2 vTexCoords;
vec4 fragColor;

uniform float iGlobalTime;
uniform vec3 iResolution;
uniform vec4 iMouse;
uniform vec4 iDate;
uniform int iFrame;

void mainImage(out vec4, in vec2);

void main() {
    mainImage(fragColor, vTexCoords * iResolution.xy);
    gl_FragColor = fragColor;
}

`;

const UNIFORM_NAMES = [
  'iGlobalTime',
  'iResolution',
  'iMouse',
  'iDate',
  'iFrame',
];

export default Ember.Component.extend({
  tagName: 'canvas',
  classNames: ['preview'],
  source: "",
  frameIndex: 0,
  startRenderTimestamp: 0,

  init() {
    this._super(...arguments);
    this.animate = this._animate.bind(this);
  },

  gl: Ember.computed('element', function() {
    let canvas = this.get('element');
    var context;

    if (canvas) {
      context = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
    }

    return context;
  }),

  program: Ember.computed('gl', 'source', function() {
    try {
      return this.programFromCompiledShadersAndUniformNames(
        this.get('gl'),
        VERTEX_SHADER,
        FRAGMENT_SHADER_PREAMBLE + this.get('source'),
        UNIFORM_NAMES
      );
    } catch(e) {
      console.error(e);
    }
  }),

  didInsertElement() {
    Ember.run.scheduleOnce('afterRender', () => {
      this.configureGl();
      window.requestAnimationFrame(this.animate);
    });
  },

  configureGl() {
    let gl = this.get('gl');
    let canvas = gl.canvas;
    let program = this.get('program');
    if (program) {
      gl.useProgram(program);

      this.configureUniforms(gl, program);
      this.configureVertices(gl, program, VERTICES);
    }
    this.set('startRenderTimestamp', Date.now());
  },

  resizeCanvas() {
    let gl = this.get('gl');
    let canvas = gl.canvas;

    if (canvas.clientWidth === canvas.width * 2 &&
        canvas.clientHeight === canvas.height * 2) {
      return;
    }

    canvas.width = canvas.clientWidth / 2;
    canvas.height = canvas.clientHeight / 2;

    gl.viewport(0, 0, canvas.width, canvas.height);
  },

  programChanged: Ember.observer('program', function() {
    Ember.run.scheduleOnce('afterRender', () => {
      this.configureGl();
    });
  }),

  configureUniforms(gl, program) {
    let canvas = gl.canvas;

    gl.uniform1f(program.uniformsCache['iGlobalTime'], (Date.now() - this.get('startRenderTimestamp')) / 1000);
    gl.uniform3fv(program.uniformsCache['iResolution'], [canvas.width, canvas.height, 1]);
    gl.uniform4fv(program.uniformsCache['iMouse'], [0, 0, 0, 0]);
    gl.uniform4fv(program.uniformsCache['iDate'], [0,0,0,0]);
    gl.uniform1i(program.uniformsCache['iFrame'], this.get('frameIndex'));
  },

  configureVertices(gl, program, vertices) {
    var positionLocation = gl.getAttribLocation(program, 'position');
    var texLocation = gl.getAttribLocation(program, 'texcoords');
    var buffer = gl.createBuffer();
    if (!buffer) { throw new Error('Failed to create buffer.'); }
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);
    gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 16, 0);
    gl.vertexAttribPointer(texLocation, 2, gl.FLOAT, false, 16, 8);
    gl.enableVertexAttribArray(positionLocation);
    gl.enableVertexAttribArray(texLocation);
  },

  clearGl() {
    let gl = this.get('gl');
    gl.disable(gl.DEPTH_TEST);
    gl.disable(gl.BLEND);
    gl.disable(gl.CULL_FACE);
    gl.clear(gl.COLOR_BUFFER_BIT);
  },

  draw() {
    let gl = this.get('gl');
    this.resizeCanvas();
    this.clearGl();
    this.configureUniforms(gl, this.get('program'));
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, NUM_VERTICES);
    this.incrementProperty('frameIndex');
  },

  _animate() {
    let gl = this.get('gl');
    if (!gl) { return; }

    this.draw();
    window.requestAnimationFrame(this.animate);
  },

  programFromCompiledShadersAndUniformNames(gl, vertexShader, fragmentShader, uniformNames) {
    var compiledVertexShader = this.compileShader(gl, gl.VERTEX_SHADER, vertexShader);
    var compiledFragmentShader = this.compileShader(gl, gl.FRAGMENT_SHADER, fragmentShader);
    var program = this.linkShader(gl, compiledVertexShader, compiledFragmentShader);
    this.cacheUniformLocations(gl, program, uniformNames);
    return program;
  },

  compileShader(gl, type, shaderSrc) {
    var shader = gl.createShader(type);
    gl.shaderSource(shader, shaderSrc);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      throw new Error(gl.getShaderInfoLog(shader));
    }

    return shader;
  },

  linkShader(gl, vertexShader, fragmentShader) {
   var program = gl.createProgram();
   gl.attachShader(program, vertexShader);
   gl.attachShader(program, fragmentShader);
   gl.linkProgram(program);

   if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
     throw new Error(gl.getProgramInfoLog(program));
   }

   return program;
 },

 cacheUniformLocations(gl, program, uniformNames) {
   let that = this;
    uniformNames.forEach(function(uniformName) {
      that.cacheUniformLocation(gl, program, uniformName);
    });
  },

  cacheUniformLocation(gl, program, label) {
  	if (!program.uniformsCache) {
  		program.uniformsCache = {};
  	}

  	program.uniformsCache[label] = gl.getUniformLocation(program, label);
  },

});
