struct VertexShader {
    id: u32,
}

impl Drop for VertexShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

impl VertexShader {
    pub fn new(vertex_shader_source: &str) -> Self {
        let vertex_shader_source = std::ffi::CString::new(vertex_shader_source).unwrap();

        let mut vertex_shader_sources = Vec::new();
        vertex_shader_sources.push(vertex_shader_source.as_ptr());

        let vertex_shader = unsafe { gl::CreateShader(gl::VERTEX_SHADER) };

        unsafe {
            gl::ShaderSource(
                vertex_shader,
                1,
                vertex_shader_sources.as_ptr(),
                std::ptr::null(),
            );

            gl::CompileShader(vertex_shader);

            let mut success = 0;

            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
            if success <= 0 {
                let mut len = 0;
                gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut len);

                let mut buffer = Vec::with_capacity(len as usize);

                gl::GetShaderInfoLog(
                    vertex_shader,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr(),
                );

                let log_info = buffer.iter().map(|&x| x as u8).collect();
                println!(
                    "Vertex shader compilation failed\n{}\n",
                    String::from_utf8_unchecked(log_info)
                );
            }
        }

        VertexShader { id: vertex_shader }
    }
}

struct FragmentShader {
    id: u32,
}

impl Drop for FragmentShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

impl FragmentShader {
    pub fn new(fragment_shader_source: &str) -> Self {
        let fragment_shader_source = std::ffi::CString::new(fragment_shader_source).unwrap();

        let mut fragment_shader_sources = Vec::new();
        fragment_shader_sources.push(fragment_shader_source.as_ptr());

        let fragment_shader = unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) };

        unsafe {
            gl::ShaderSource(
                fragment_shader,
                1,
                fragment_shader_sources.as_ptr(),
                std::ptr::null(),
            );

            gl::CompileShader(fragment_shader);

            let mut success = 0;

            gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
            if success <= 0 {
                let mut len = 0;
                gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut len);

                let mut buffer = Vec::with_capacity(len as usize);

                gl::GetShaderInfoLog(
                    fragment_shader,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr(),
                );

                let log_info = buffer.iter().map(|&x| x as u8).collect();
                println!(
                    "Fragment shader compilation failed\n{}\n",
                    String::from_utf8_unchecked(log_info)
                );
            }

            FragmentShader {
                id: fragment_shader,
            }
        }
    }
}

pub struct ShaderProgram {
    pub id: u32,
}

impl ShaderProgram {
    pub fn new(vertex_shader_source: &str, fragment_shader_source: &str) -> Self {
        let vs = VertexShader::new(vertex_shader_source);
        let fs = FragmentShader::new(fragment_shader_source);

        let shader_program = unsafe { gl::CreateProgram() };

        unsafe {
            gl::AttachShader(shader_program, vs.id);
            gl::AttachShader(shader_program, fs.id);
            gl::LinkProgram(shader_program);

            let mut success = 0;

            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
            if success <= 0 {
                let mut len = 0;
                gl::GetProgramiv(shader_program, gl::INFO_LOG_LENGTH, &mut len);

                let mut buffer = Vec::with_capacity(len as usize);

                gl::GetProgramInfoLog(
                    shader_program,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr(),
                );

                let log_info = buffer.iter().map(|&x| x as u8).collect();
                println!(
                    "Shader program shader compilation failed\n{}\n",
                    String::from_utf8_unchecked(log_info)
                );
            }

            ShaderProgram { id: shader_program }
        }
    }
}
