/* Draku hero — GLSL: dark fur-like fog lit from below by crimson wing-light,
   with drifting embers. Falls back silently if WebGL is unavailable. */
(function () {
  var canvas = document.getElementById('embers');
  if (!canvas) return;
  var gl = canvas.getContext('webgl', { antialias: false, premultipliedAlpha: false });
  if (!gl) return;

  var vs = 'attribute vec2 p;void main(){gl_Position=vec4(p,0.,1.);}';
  var fs = [
    'precision mediump float;',
    'uniform vec2 r;uniform float t;uniform vec2 m;',
    'float h(vec2 p){return fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453);}',
    'float n(vec2 p){vec2 i=floor(p),f=fract(p);f=f*f*(3.-2.*f);',
    ' return mix(mix(h(i),h(i+vec2(1,0)),f.x),mix(h(i+vec2(0,1)),h(i+vec2(1,1)),f.x),f.y);}',
    'float fbm(vec2 p){float v=0.,a=.5;for(int i=0;i<5;i++){v+=a*n(p);p=p*2.03+vec2(1.7,9.2);a*=.5;}return v;}',
    'void main(){',
    ' vec2 uv=gl_FragCoord.xy/r;vec2 q=uv;q.x*=r.x/r.y;',
    ' vec2 mm=(m-.5)*.15;',
    ' float f=fbm(q*2.2+vec2(0.,-t*.05)+mm+fbm(q*3.+t*.03));',
    ' vec3 fur=vec3(.031,.027,.031);',
    ' vec3 wing=vec3(.52,.12,.16);',
    ' vec3 glow=vec3(.76,.25,.31);',
    ' float low=smoothstep(.95,0.,uv.y);',
    ' vec3 c=fur+wing*pow(f,2.6)*low*1.1;',
    // embers
    ' float e=0.;',
    ' for(int i=0;i<3;i++){float s=float(i)*7.3+1.;',
    '  vec2 g=q*vec2(14.+s,9.+s)+vec2(0.,t*(.35+float(i)*.18));',
    '  vec2 id=floor(g);vec2 fr=fract(g)-.5;float rn=h(id+s);',
    '  if(rn>.93){vec2 o=vec2(sin(t+rn*40.)*.25,0.);float d=length(fr-o);',
    '   e+=smoothstep(.07,0.,d)*(.5+.5*sin(t*3.+rn*60.));}}',
    ' c+=glow*e*low*1.3;',
    // vignette
    ' float v=smoothstep(1.25,.25,length((uv-vec2(.5,.45))*vec2(1.2,1.)));',
    ' c*=mix(.35,1.,v);',
    ' gl_FragColor=vec4(c,1.);}'
  ].join('\n');

  function sh(type, src) {
    var s = gl.createShader(type); gl.shaderSource(s, src); gl.compileShader(s);
    if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) { console.warn(gl.getShaderInfoLog(s)); return null; }
    return s;
  }
  var v = sh(gl.VERTEX_SHADER, vs), f = sh(gl.FRAGMENT_SHADER, fs);
  if (!v || !f) return;
  var pr = gl.createProgram(); gl.attachShader(pr, v); gl.attachShader(pr, f); gl.linkProgram(pr);
  gl.useProgram(pr);
  var b = gl.createBuffer(); gl.bindBuffer(gl.ARRAY_BUFFER, b);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
  var loc = gl.getAttribLocation(pr, 'p'); gl.enableVertexAttribArray(loc); gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 0, 0);
  var uR = gl.getUniformLocation(pr, 'r'), uT = gl.getUniformLocation(pr, 't'), uM = gl.getUniformLocation(pr, 'm');

  var mouse = [0.5, 0.5], target = [0.5, 0.5];
  window.addEventListener('pointermove', function (e) { target = [e.clientX / innerWidth, 1 - e.clientY / innerHeight]; }, { passive: true });

  function resize() {
    var dpr = Math.min(window.devicePixelRatio || 1, 1.5) * 0.6; // soft & cheap
    canvas.width = Math.floor(canvas.clientWidth * dpr);
    canvas.height = Math.floor(canvas.clientHeight * dpr);
    gl.viewport(0, 0, canvas.width, canvas.height);
  }
  resize(); window.addEventListener('resize', resize);

  var reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  var visible = true;
  new IntersectionObserver(function (en) { visible = en[0].isIntersecting; }).observe(canvas);
  var t0 = performance.now();
  function frame(now) {
    if (visible) {
      mouse[0] += (target[0] - mouse[0]) * .05; mouse[1] += (target[1] - mouse[1]) * .05;
      gl.uniform2f(uR, canvas.width, canvas.height);
      gl.uniform1f(uT, reduce ? 4 : (now - t0) / 1000);
      gl.uniform2f(uM, mouse[0], mouse[1]);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
    }
    if (!reduce) requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
})();
