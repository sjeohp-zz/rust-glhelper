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

pub static mut LINE_DATA: [GLfloat; 10000] = [0.; 10000];

pub fn add_path_line(path: &Vec<(f32, f32)>, path_edges: usize, line_program: GLuint, line_vao: GLuint, line_vbo: GLuint)
{
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

pub fn compile_shader(filename: &str, shader_type: GLenum) -> GLuint
{

	let file = match File::open(filename) {
		Ok(file) => file,
		Err(..) => panic!("opening {}", filename)
	};
	let mut reader = BufReader::new(&file);
	let src = &mut String::new();
	match reader.read_to_string(src) {
		Ok(_) => {},
		Err(..) => panic!("reading {}", filename)
	}

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

pub fn load_program(vfilename: &str, ffilename: &str, shaders: &mut Vec<GLuint>) -> GLuint
{
	let vs = compile_shader(vfilename, gl::VERTEX_SHADER);
	let fs = compile_shader(ffilename, gl::FRAGMENT_SHADER);

	let program = link_program(vs, fs);

	shaders.push(vs);
	shaders.push(fs);

	program
}

pub fn load_program_with_shader_outs(vfilename: &str, vout: &mut GLuint, ffilename: &str, fout: &mut GLuint) -> GLuint
{
	let vs = compile_shader(vfilename, gl::VERTEX_SHADER);
	let fs = compile_shader(ffilename, gl::FRAGMENT_SHADER);

	let program = link_program(vs, fs);

	*vout = vs;
	*fout = fs;

	program
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
