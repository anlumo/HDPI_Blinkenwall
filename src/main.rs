#[macro_use] extern crate log;
extern crate env_logger;

#[macro_use]
extern crate glium;
extern crate chrono;
use glium::glutin;
use glium::DisplayBuild;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

mod shadertoy;
use shadertoy::ShaderToy;

mod server;
mod database;

use std::sync::mpsc;

const FRAGMENT_SHADER: &'static str = r##"
/*TEXTURE_FORMAT:OutputFormat(1280, 720, GL_RGB, GL_UNSIGNED_BYTE, GL_NEAREST, GL_NEAREST)

CALL:FORMAT_TO_CONSTANT(OutputFormat, OutputFormatSize)

SHADER_SOURCE:HackathonShader
{*/
	//#version 130
	//#define SINGLE_UNIT	// Single unit, for tests purpose.
	//#define MULTI_SIMPLE	// Showing the repitition bug, comment to see the solved version.
	#define FAST_METHOD		// If this is commented, the method used is exact. Uncomment for speed.
	//#define NO_EFFECTS	// Remove the image effects.
	#define INFINITE		// Infinite animation.

	//INSERT(OutputFormatSize)

	/*uniform vec3	eyePos		= vec3(0, 0, 4),
					eyeTarget	= vec3(0, 0, 0),
					lightDir	= vec3(1, 0, 0);
	uniform	float	focalLength	= 0.5,
					lightFlux	= 1.0;*/
	vec3		eyePos		= vec3(4.0, 3.0, 4.0),
        		eyeTarget	= vec3(0.0, 0.0, 0.0);
	const vec3	lightDir	= vec3(1.0, 0.0, 0.2);
	const float	focalLength	= 0.5,
				lightFlux	= 4.0,
                volumeSide 	= 10.0,
        		duration	= 30.0;

	float blockHash(vec3 p)
	{
		//return sin(123.0*p.x)*sin(456.0*p.y)*sin(789.0*p.z)*241432439.0;
        return sin(123.0*p.x)*sin(456.0*p.y)*sin(789.0*p.z)*2769.32;
	}

	float smin(float a, float b)
	{
		const float k = 0.1;
		float h = clamp(0.5+0.5*(b-a)/k, 0.0, 1.0);
		return mix(b, a, h) - k*h*(1.0-h);
	}

	float absSq(float x)
    {
        return x*x;
    }

	float getUnitDistance(float idx, vec3 p)
	{
		vec3 	bumps1 		= sin(idx*vec3(2.542, 1.564, 3.342))+1.0,
				bumps2		= sin(idx*vec3(-1.432, 8.788, 9.453))+1.0,
				translation	= sin(idx*vec3(0.345, 8.342, 3.324))/1.5;
        bumps1 = max(abs(bumps1), vec3(1.0))*sign(bumps1); // avoid 0.0, avoid the spheres...
        bumps2 = max(abs(bumps2), vec3(1.0))*sign(bumps2);

		p += sin(idx*vec3(0.234, 0.736, 0.213))*3.0;

		float	s1 	= length(p) + sin(bumps1.x*p.x)*sin(bumps1.y*p.y)*sin(bumps1.z*p.z)/1.5 + sin(bumps1.x*p.x*4.0)*sin(bumps1.y*p.y*4.0)*sin(bumps1.z*p.z*4.0)/16.0 - 1.0,
				s2	= distance(p, translation) + sin(bumps2.x*p.x)*sin(bumps2.y*p.y)*sin(bumps2.z*p.z)/1.5  + sin(bumps2.x*p.x*4.0)*sin(bumps2.y*p.y*4.0)*sin(bumps2.z*p.z*4.0)/16.0 - 1.0;
		return  smin(s1, s2);
    }

	/*float getUnitDistance(vec3 block, vec3 p)
	{
		vec3 	bumps1 		= fract(block*vec3(0.2342, 0.4564, 0.3342))*2.0,
				bumps2		= fract(block*vec3(-0.7432, 0.8788, 0.9453))*2.0,
				translation	= (fract(block*vec3(0.3450, 0.8342, -0.8324))*2.0-1.0)/1.5;
        bumps1 = max(abs(bumps1), vec3(1.0))*sign(bumps1);
        bumps2 = max(abs(bumps2), vec3(1.0))*sign(bumps2);

		p = p + (fract(block*vec3(0.2334, 0.5365, 0.4353))*2.0-1.0)*4.0;

		float	s1 	= length(p) + sin(bumps1.x*p.x)*sin(bumps1.y*p.y)*sin(bumps1.z*p.z)/1.5 + sin(bumps1.x*p.x*4.0)*sin(bumps1.y*p.y*4.0)*sin(bumps1.z*p.z*4.0)/16.0 - 1.0,
				s2	= distance(p, translation) + sin(bumps2.x*p.x)*sin(bumps2.y*p.y)*sin(bumps2.z*p.z)/1.5  + sin(bumps2.x*p.x*4.0)*sin(bumps2.y*p.y*4.0)*sin(bumps2.z*p.z*4.0)/16.0 - 1.0;

		return  smin(s1, s2);
    }*/

	float getUnitDistanceSimple(vec3 p)
    {
        return length(p)-4.8; // The coefficient is a bit ad-hoc here. The goal is to quickly catch a "leaking ray".
    }

	vec2 sceneMap(vec3 p, vec3 dir)
	{
		#ifdef SINGLE_UNIT
        	const float bidx = 43578.4534;
			return vec2(getUnitDistance(bidx, p), 1.0+abs(bidx));
		#else
        #ifdef MULTI_SIMPLE
        	vec3	m0 = mod(p+volumeSide/2.0, volumeSide)-volumeSide/2.0;
        	vec3	v = floor((p+volumeSide/2.0)/volumeSide)-volumeSide/2.0;
			float	bidx0 = blockHash(v);

        	return vec2(getUnitDistance(bidx0, m0), 1.0+abs(bidx0));

        	/*vec3	m0 = mod(p+volumeSide/2.0, volumeSide)-volumeSide/2.0;
        	vec3	block = floor(p/volumeSide) + vec3(24.32, 32.4324, 63.6548);
        	return vec2(getUnitDistance(block, m0), 1.0);*/
        #else
        #ifdef FAST_METHOD
        	vec3	m0 = mod(p+volumeSide/2.0, volumeSide)-volumeSide/2.0,
                	m1 = m0 - sign(vec3(dir.x, 0.0, 0.0)) * volumeSide,
					m2 = m0 - sign(vec3(0.0, dir.y, 0.0)) * volumeSide,
					m3 = m0 - sign(vec3(0.0, 0.0, dir.z)) * volumeSide,
					m4 = m0 - sign(vec3(dir.x, dir.y, 0.0)) * volumeSide,
					m5 = m0 - sign(vec3(0.0, dir.y, dir.z)) * volumeSide,
					m6 = m0 - sign(vec3(dir.x, 0.0, dir.z)) * volumeSide,
					m7 = m0 - sign(vec3(dir.x, dir.y, dir.z)) * volumeSide;
        	vec3	v = floor((p+volumeSide/2.0)/volumeSide)-volumeSide/2.0;
            float	bidx0 = blockHash(v);
        	float 	d = min(getUnitDistance(bidx0, m0),
						min(getUnitDistanceSimple(m1),
						min(getUnitDistanceSimple(m2),
						min(getUnitDistanceSimple(m3),
						min(getUnitDistanceSimple(m4),
						min(getUnitDistanceSimple(m5),
						min(getUnitDistanceSimple(m6),
							getUnitDistanceSimple(m7) )))))));
        	return vec2(d, 1.0+abs(bidx0));
        #else
			vec3	m0 = mod(p+volumeSide/2.0, volumeSide)-volumeSide/2.0,
					m1 = m0 - sign(vec3(dir.x, 0.0, 0.0)) * volumeSide,
					m2 = m0 - sign(vec3(0.0, dir.y, 0.0)) * volumeSide,
					m3 = m0 - sign(vec3(0.0, 0.0, dir.z)) * volumeSide,
					m4 = m0 - sign(vec3(dir.x, dir.y, 0.0)) * volumeSide,
					m5 = m0 - sign(vec3(0.0, dir.y, dir.z)) * volumeSide,
					m6 = m0 - sign(vec3(dir.x, 0.0, dir.z)) * volumeSide,
					m7 = m0 - sign(vec3(dir.x, dir.y, dir.z)) * volumeSide;
			vec3	v = floor((p+volumeSide/2.0)/volumeSide)-volumeSide/2.0;
			float	bidx0 = blockHash(v),
					bidx1 = blockHash(v + vec3(sign(dir.x), 0.0, 0.0)),
					bidx2 = blockHash(v + vec3(0.0, sign(dir.y), 0.0)),
					bidx3 = blockHash(v + vec3(0.0, 0.0, sign(dir.z))),
					bidx4 = blockHash(v + vec3(sign(dir.x), sign(dir.y), 0.0)),
					bidx5 = blockHash(v + vec3(0.0, sign(dir.y), sign(dir.z))),
					bidx6 = blockHash(v + vec3(sign(dir.x), 0.0, sign(dir.z))),
					bidx7 = blockHash(v + vec3(sign(dir.x), sign(dir.y), sign(dir.z)));
			float 	d = min(getUnitDistance(bidx0, m0),
						min(getUnitDistance(bidx1, m1),
						min(getUnitDistance(bidx2, m2),
						min(getUnitDistance(bidx3, m3),
						min(getUnitDistance(bidx4, m4),
						min(getUnitDistance(bidx5, m5),
						min(getUnitDistance(bidx6, m6),
							getUnitDistance(bidx7, m7) )))))));
        	/*vec3	m0 = mod(p+volumeSide/2.0, volumeSide)-volumeSide/2.0,
					m1 = m0 - sign(vec3(dir.x, 0.0, 0.0)) * volumeSide,
					m2 = m0 - sign(vec3(0.0, dir.y, 0.0)) * volumeSide,
					m3 = m0 - sign(vec3(0.0, 0.0, dir.z)) * volumeSide,
					m4 = m0 - sign(vec3(dir.x, dir.y, 0.0)) * volumeSide,
					m5 = m0 - sign(vec3(0.0, dir.y, dir.z)) * volumeSide,
					m6 = m0 - sign(vec3(dir.x, 0.0, dir.z)) * volumeSide,
					m7 = m0 - sign(vec3(dir.x, dir.y, dir.z)) * volumeSide;
			vec3	v = floor((p+volumeSide/2.0)/volumeSide)-volumeSide/2.0,
        			block1 = v + vec3(sign(dir.x), 0.0, 0.0),
					block2 = v + vec3(0.0, sign(dir.y), 0.0),
					block3 = v + vec3(0.0, 0.0, sign(dir.z)),
					block4 = v + vec3(sign(dir.x), sign(dir.y), 0.0),
					block5 = v + vec3(0.0, sign(dir.y), sign(dir.z)),
					block6 = v + vec3(sign(dir.x), 0.0, sign(dir.z)),
					block7 = v + vec3(sign(dir.x), sign(dir.y), sign(dir.z));
        	float 	d = min(getUnitDistance(v, m0),
						min(getUnitDistance(block1, m1),
						min(getUnitDistance(block2, m2),
						min(getUnitDistance(block3, m3),
						min(getUnitDistance(block4, m4),
						min(getUnitDistance(block5, m5),
						min(getUnitDistance(block6, m6),
							getUnitDistance(block7, m7) )))))));*/
			return vec2(d, 1.0+abs(bidx0));
        #endif
		#endif
        #endif
	}

	vec4 renderMaterial(vec2 fragCoord, vec3 p, vec3 n, float m)
	{
        float d = distance(eyePos, p);
		vec4 c = vec4(0.0, 0.0, 0.0, 1.0);

		if(m>=1.0)
			c.rgb = vec3(1.0, 1.0, 1.0) * (0.03 + max(dot(normalize(lightDir), n), 0.0) * max(dot(normalize(eyePos-p), n), 0.0) * lightFlux);

        #ifndef NO_EFFECTS
        if(floor(mod(m,10.0))==0.0 && d<50.0)
			c.rg += vec2(1.0, 0.25)*max(0.9 + sin(fragCoord.y/iResolution.y*100.0-iGlobalTime*10.0)/2.0, 0.0);
        #endif

        const float cFloor = 0.01,
            		tau = 20.0;

        c.rgb = c.rgb*exp(-d/tau)*(1.0-cFloor)+cFloor*(1.0-exp(-d/tau));

		return c;
	}

	// Core of the Ray-marcher :
	mat3 computeCameraMatrix(in vec3 p, in vec3 target, float roll)
	{
		vec3 	vForward = normalize(target-p),
				vUpAlign = vec3(sin(roll), cos(roll), 0.0),
				vLeftReal = normalize(cross(vForward, vUpAlign)),
				vUpReal = normalize(cross(vLeftReal, vForward));
	   	return mat3(vLeftReal, vUpReal, vForward);
	}

	vec3 castRay(in vec3 rayOrigin, in vec3 rayDirection)
	{
		const int 	numSteps = 128;
		const float	dMin = 0.0,
					dNear = 0.000001,
					dMax = 100.0;
		float		d = dMin,
					dLast = dMin,
			m = -1.0;
		for(int i=0; i<numSteps; i++)
		{
			vec3 p = rayOrigin+rayDirection*d;
			vec2 res = sceneMap(p, rayDirection);

			dLast = res.x;
			d += res.x;
			m = res.y;

			if(res.x<dNear || d>dMax)
				break;
		}
		if(d>dMax)
			m = -1.0;

		return vec3(d, m, dLast);
	}

	vec3 calcNormal(in vec3 pos, in vec3 dir)
	{
		const vec3 eps = vec3(0.001, 0.0, 0.0);
		vec3 n = vec3(	sceneMap(pos+eps.xyy, dir).x - sceneMap(pos-eps.xyy, dir).x,
						sceneMap(pos+eps.yxy, dir).x - sceneMap(pos-eps.yxy, dir).x,
						sceneMap(pos+eps.yyx, dir).x - sceneMap(pos-eps.yyx, dir).x );
		return normalize(n);
	}

	vec4 renderScene(vec2 fragCoord, const ivec2 formatSize, const vec3 eyePos, const vec3 eyeTarget, const float focalLength)
	{
		mat3 camera = computeCameraMatrix(eyePos, eyeTarget, 0.0);
		vec2 o = (fragCoord - vec2(formatSize)/2.0)/max(float(formatSize.x),float(formatSize.y));
		vec3 rayOrigin = vec3(o, 0.0) + eyePos,
		     rayDirection = normalize(camera*vec3(o, focalLength));
		vec3 res = castRay(rayOrigin, rayDirection);
		vec3 p = rayOrigin + rayDirection * res.x;
		vec3 n = calcNormal(p, rayDirection);
		return renderMaterial(fragCoord, p, n, res.y);
	}

	void mainImage(out vec4 fragColor, in vec2 fragCoord)
	{
        eyePos = eyePos + vec3(sin(iGlobalTime/10.0)/2.0, sin(iGlobalTime/10.0)/2.0, iGlobalTime*1.4);
        eyeTarget = eyePos + vec3(sin(iGlobalTime/2.0)/10.0, 0.0, 1);

        #ifndef NO_EFFECTS
        	vec2 fragCoordMod = fragCoord + sin(iGlobalTime/2.0+fragCoord.y/5.0)*100.0*exp(-absSq(fragCoord.y/iResolution.y-sin(iGlobalTime)-0.5)*800.0);
        #else
        	vec2 fragCoordMod = fragCoord;
        #endif

		// Render the scene :
        vec4 c = renderScene(fragCoordMod, ivec2(iResolution.xy), eyePos, eyeTarget, focalLength);

        // Effects :
        #ifndef NO_EFFECTS
        	c.rgb *= max(cos(3.14*length(fragCoord.xy/iResolution.xy - vec2(0.5,0.5))*0.85), 0.0);
    		c.rgb *= (iGlobalTime<=0.0) ? 1.0 : (1.0-exp(-iGlobalTime/4.0));
        	c.rgb *= (floor(mod(fragCoord.y,4.0))<=1.0) ? 1.5 : 1.0;

        	#ifndef INFINITE
        	if(iGlobalTime>duration)
            {
                float 	y = abs(fragCoord.y/iResolution.y - 0.5),
               			ylim = exp(-(iGlobalTime-duration)*10.0);
           		c.rgb *= float(y<ylim) + 100.0*exp(-absSq(y-ylim)*(10000.0*iGlobalTime));
                c.rgb *= float(iGlobalTime<duration + 0.5);
            }
        	#endif

       	 	// hud 1 :
       		vec2 v = fragCoord/max(iResolution.x,iResolution.y);
        	c.r += 0.4*smoothstep(0.012, 0.010, distance(v,vec2(0.95, 0.53)))*float(mod(iGlobalTime,2.0)<=1.0);

        	// hud 2 :
        	/*float p1 = max(abs(v.x-0.5), abs(v.y-0.05)*3.0);
        	c.r += 0.4*float(p1<0.05 && p1>0.04)*float(mod(iGlobalTime,0.5)<=0.25);
        	float p2 = max(abs(v.x-0.555)*3.0, abs(v.y-0.05));
        	c.r += 0.4*float(p2<0.01)*float(mod(iGlobalTime,0.5)<=0.25);*/
    	#endif

        fragColor = vec4(pow(c.rgb, vec3(1.0, 1.0, 1.0)/2.2), 1.0);
    }
/*}

FILTER_LAYOUT:HackathonFilter(OutputFormat, HackathonShader)

PIPELINE_MAIN:HackathonPipeline
{
	OUTPUT_PORTS(outputTexture)

	FILTER_INSTANCE:HackathonFilter
}
*/
"##;

const DISPLAY_WIDTH: u32 = 192;
const DISPLAY_HEIGHT: u32 = 144;

fn main() {
    env_logger::init().unwrap();
    let display = glutin::WindowBuilder::new()
        .with_depth_buffer(24)
        .with_fullscreen(glutin::get_primary_monitor())
        .with_vsync()
        .build_glium()
        .unwrap();
    display.get_window().unwrap().set_inner_size(DISPLAY_WIDTH, DISPLAY_HEIGHT);

    let mut shadertoy = ShaderToy::new(&display, FRAGMENT_SHADER);

    let database = database::Database::new();
    let (server_thread, command_receiver) = server::open_server(1337);

    loop {
        shadertoy.step(&display);
        match command_receiver.try_recv() {
            Ok(message) => {
                let (cmd, resp) = message;
                match cmd {
                    server::Command::List => resp.send_list(database.list()),
                    server::Command::Read(key) => resp.send_error(404, "Not implemented"),
                    server::Command::Write(key, content) => resp.send_error(404, "Not implemented"),
                    server::Command::Create(content) => resp.send_ok(),
                    server::Command::Activate(key) => resp.send_ok(),
                }.unwrap();
            },
            Err(err) => match err {
                mpsc::TryRecvError::Empty => (),
                mpsc::TryRecvError::Disconnected => break,
            }
        }
    }
    let _ = server_thread.join().unwrap();
}
