#![crate_name = "glhelper"]
#![crate_type = "lib"]

extern crate gl;

use self::gl::types::{GLenum, GLint, GLuint, GLchar, GLfloat, GLsizeiptr};

use std::io::{BufReader, Read};
use std::fs::File;
use std::ptr;
use std::str;
use std::ffi::CString;
use std::mem;

pub static QUAD_DATA: [GLfloat; 20] =
[
    -1.,  1., 0.0, 0.0, 0.0,
     1.,  1., 0.0, 1.0, 0.0,
    -1., -1., 0.0, 0.0, 1.0,
	 1., -1., 0.0, 1.0, 1.0
];

pub fn compile_shader(src: &'static str, shader_type: GLenum) -> GLuint
{
    let shader;
    unsafe {
        shader = gl::CreateShader(shader_type);
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(&buf).ok().expect("ShaderInfoLog invalid UTF8"));
        }
    }
    shader
}

pub fn link_program(vs: GLuint, fs: GLuint) -> GLuint
{
	unsafe {
	    let program = gl::CreateProgram();
	    gl::AttachShader(program, vs);
	    gl::AttachShader(program, fs);
	    gl::LinkProgram(program);
	    let mut status = gl::FALSE as GLint;
	    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
	    if status != (gl::TRUE as GLint) {
	        let mut len: GLint = 0;
	        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
	        let mut buf = Vec::with_capacity(len as usize);
	        buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
	        gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
	        panic!("{}", str::from_utf8(&buf).ok().expect("ProgramInfoLog invalid UTF8"));
	    }
	    program
	}
}

pub fn load_program(vsrc: &'static str, fsrc: &'static str, shaders: &mut Vec<GLuint>) -> GLuint
{
	let vs = compile_shader(vsrc, gl::VERTEX_SHADER);
	let fs = compile_shader(fsrc, gl::FRAGMENT_SHADER);

	let program = link_program(vs, fs);

	shaders.push(vs);
	shaders.push(fs);

	program
}

pub fn load_program_with_shader_outs(vsrc: &'static str, vout: &mut GLuint, fsrc: &'static str, fout: &mut GLuint) -> GLuint
{
	let vs = compile_shader(vsrc, gl::VERTEX_SHADER);
	let fs = compile_shader(fsrc, gl::FRAGMENT_SHADER);

	let program = link_program(vs, fs);

	*vout = vs;
	*fout = fs;

	program
}

pub fn check_gl_error(file: &'static str, line: u32)
{
	unsafe
	{
		loop
		{
			let err_str: &'static str = match gl::GetError()
			{
				gl::NO_ERROR => { break },
				gl::INVALID_OPERATION => { "INVALID_OPERATION" },
				gl::INVALID_ENUM => { "INVALID_ENUM" },
				gl::INVALID_VALUE => { "INVALID_VALUE" },
				gl::OUT_OF_MEMORY => { "OUT_OF_MEMROY" },
				gl::INVALID_FRAMEBUFFER_OPERATION => { "INVALID_FRAMEBUFFER_OPERATION" }
				_ => { "" }
			};
			println!("GL_{}: {}, {}", err_str, file, line);
		}
	}
}

// pub static mut LINE_DATA: [GLfloat; 10000] = [0.; 10000];

pub fn add_path_line(path: &[(f32, f32)], path_edges: usize, line_program: GLuint, line_vao: GLuint, line_vbo: GLuint)
{
	let mut LINE_DATA: Vec<f32> = vec![0.0; path_edges];
	let mut offset: usize = 0;
	let stride = 16;

	unsafe
	{
		for i in 0..path_edges
		{
			let normalx = path[i+1].1 - path[i].1;
			let normaly = path[i].0 - path[i+1].0;
			LINE_DATA[offset+0] = path[i].0;
			LINE_DATA[offset+1] = path[i].1;
			LINE_DATA[offset+2] = -normalx;
			LINE_DATA[offset+3] = -normaly;
			LINE_DATA[offset+4] = path[i].0;
			LINE_DATA[offset+5] = path[i].1;
			LINE_DATA[offset+6] = normalx;
			LINE_DATA[offset+7] = normaly;
			LINE_DATA[offset+8] = path[i+1].0;
			LINE_DATA[offset+9] = path[i+1].1;
			LINE_DATA[offset+10] = -normalx;
			LINE_DATA[offset+11] = -normaly;
			LINE_DATA[offset+12] = path[i+1].0;
			LINE_DATA[offset+13] = path[i+1].1;
			LINE_DATA[offset+14] = normalx;
			LINE_DATA[offset+15] = normaly;
			offset += stride;
		}
		gl::UseProgram(line_program);
		gl::BindVertexArray(line_vao);
		gl::BindBuffer(gl::ARRAY_BUFFER, line_vbo);
		gl::BufferSubData(gl::ARRAY_BUFFER, 0, (path_edges * stride * mem::size_of::<GLfloat>()) as GLsizeiptr as isize, mem::transmute(&LINE_DATA[0]));
		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		gl::BindVertexArray(0);
		gl::UseProgram(0);
	}
}


#[cfg(test)]
mod tests {
	extern crate sdl2;
	use self::sdl2::video::{GLProfile};
	use self::sdl2::rect::Rect;
	use self::sdl2::event::{Event};

	use std::os::raw::c_void;

	use super::*;	

    #[test]
    fn it_works() {

    	let sdl = sdl2::init().unwrap();
    	let video_subsystem = sdl.video().unwrap();
    	let gl_attr = video_subsystem.gl_attr();

    	gl_attr.set_context_flags().debug().set();
    	gl_attr.set_context_version(3, 3);
    	gl_attr.set_context_profile(GLProfile::Core);
    	gl_attr.set_multisample_buffers(1);
    	gl_attr.set_multisample_samples(4);
    	gl_attr.set_double_buffer(true);
    	gl_attr.set_depth_size(24);

    	let window_bounds = Rect::new(100, 100, 500, 500);
    	let mut window = video_subsystem.window(
    	    "glhelper test", 
    	    window_bounds.width(), 
    	    window_bounds.height())
    	.position(
    	    window_bounds.x(), 
    	    window_bounds.y())
    	.opengl().build().unwrap();

    	let context = match window.gl_create_context() {
    	    Ok(res) => res,
    	    Err(..) => panic!("Could not open vert shader")
    	};
    	match window.gl_make_current(&context) {
    	    Ok(_) => {},
    	    Err(..) => panic!("setting current context")
    	}

    	// assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    	// assert_eq!(gl_attr.context_version(), (3, 3));

    	gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const c_void);

    	// video_subsystem.gl_set_swap_interval(1);

    	let mut shaders: Vec<GLuint> = vec![];
    	let vsrc = include_str!("../shaders/line.vert.glsl");
    	let fsrc = include_str!("../shaders/line.frag.glsl");
    	let line_program = load_program(
    	    vsrc, 
    	    fsrc, 
    	    &mut shaders);
    }
}
