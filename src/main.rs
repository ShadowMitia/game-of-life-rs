use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

mod Shaders;

struct GameOfLife<'a> {
    width: u32,
    height: u32,
    simulation: Vec<&'a bool>,
}

fn index<T>(x: T, y: T, width: T) -> T
where
    T: std::ops::Mul<Output = T> + std::ops::Add<Output = T>,
{
    y * width + x
}

impl GameOfLife<'_> {
    fn new(width: u32, height: u32) -> Self {
        GameOfLife {
            width,
            height,
            simulation: vec![&false; (width * height) as usize],
        }
    }

    fn at(&self, x: u32, y: u32) -> &bool {
        self.simulation[index(x, y, self.width) as usize]
    }

    fn simulate(&self) -> Self {
        let mut new_simulation = vec![&false; (self.width * self.height) as usize];

        let mut neighbor_count: Vec<i32> = vec![0; (self.width * self.height) as usize];

        for i in 0..self.width as i32 {
            for j in 0..self.height as i32 {
                for ni in -1 as i32..=1 {
                    for nj in -1 as i32..=1 {
                        if ni == 0 && nj == 0 {
                            continue;
                        }
                        let ni = if i + ni < 0 {
                            (self.width - 1) as i32
                        } else if i + ni >= self.width as i32 {
                            0 as i32
                        } else {
                            i + ni
                        };

                        let nj = if j + nj < 0 {
                            (self.height - 1) as i32
                        } else if j + nj >= self.height as i32 {
                            0 as i32
                        } else {
                            j + nj
                        };

                        let active = self.simulation[index(ni, nj, self.width as i32) as usize];
                        if *active {
                            let n = neighbor_count[index(i, j, self.width as i32) as usize];
                            neighbor_count[index(i, j, self.width as i32) as usize] = n + 1;
                        }
                    }
                }
            }
        }

        for i in 0..self.width as i32 {
            for j in 0..self.height as i32 {
                let index = index(i, j, self.width as i32);

                let is_active = self.simulation[index as usize];
                let n_count = neighbor_count[index as usize];

                new_simulation[index as usize] = match (is_active, n_count) {
                    (true, 2) => &true,
                    (_, 3) => &true,
                    _ => &false,
                }
            }
        }

        GameOfLife {
            width: self.width,
            height: self.height,
            simulation: new_simulation,
        }
    }
}

pub fn main() -> Result<(), String> {
    let mut rng = rand::thread_rng();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("Game of life", 800, 600)
        .position_centered()
        .opengl()
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let _gl = window.gl_create_context().unwrap();

    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    unsafe {
        gl::Viewport(0, 0, 800, 600);
    }

    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    let mut event_pump = sdl_context.event_pump().unwrap();

    let rectangle: Vec<f32> = vec![
        -1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, -1.0,
        0.0, 1.0, 0.0,
    ];

    let indices = vec![0, 1, 2, 2, 1, 3];

    let vao = unsafe {
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        vao
    };

    unsafe {
        gl::BindVertexArray(vao);

        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            ((rectangle.len()) * std::mem::size_of::<f32>()) as isize,
            rectangle.as_ptr() as *const std::ffi::c_void,
            gl::STATIC_DRAW,
        );

        let mut ebo = 0;
        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<f32>()) as isize,
            indices.as_ptr() as *const std::ffi::c_void,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (5 * std::mem::size_of::<f32>()) as i32,
            std::ptr::null::<std::ffi::c_void>(),
        );

        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            (5 * std::mem::size_of::<f32>()) as i32,
            std::ptr::null::<std::ffi::c_void>(),
        );

        gl::EnableVertexAttribArray(1);
    }

    let vertex_shader_source = r"
        #version 330 core
        layout(location = 0) in vec3 aPos;
        layout(location = 1) in vec2 aTexCoord;
        
        out vec2 TexCoord;

        void main()
        {
            gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0f);

            TexCoord = aTexCoord;
        }
    ";

    let fragment_shader_source = r"
        #version 330 core
        out vec4 FragColor;

        in vec2 TexCoord;

        uniform sampler2D tex;

        void main() {
            FragColor = texture(tex, TexCoord);
        }
    ";

    let shader_program = Shaders::ShaderProgram::new(vertex_shader_source, fragment_shader_source);

    unsafe {
        let t = std::ffi::CString::new("tex").unwrap();

        gl::Uniform1i(gl::GetUniformLocation(shader_program.id, t.as_ptr()), 0);
    }

    let mut game_of_life = GameOfLife::new(100, 100);

    game_of_life.simulation = game_of_life
        .simulation
        .iter()
        .map(|_| {
            if rng.gen::<f32>() < 0.3 {
                &true
            } else {
                &false
            }
        })
        .collect();

    'running: loop {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        let mut is_spaced_pressed = false;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => is_spaced_pressed = true,
                _ => {}
            }
        }

        // UPDATE
        if is_spaced_pressed {
            game_of_life = game_of_life.simulate();
        }

        // RENDERING

        let simulation_rgb: Vec<u8> = game_of_life
            .simulation
            .iter()
            .map(|&value| {
                if *value {
                    vec![0, 0, 0]
                } else {
                    vec![255, 255, 255]
                }
            })
            .flatten()
            .collect();

        let image: image::RgbImage =
            image::ImageBuffer::from_raw(game_of_life.width, game_of_life.height, simulation_rgb)
                .unwrap();

        let texture = unsafe {
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            // set the texture wrapping parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            // set texture filtering parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                game_of_life.width as i32,
                game_of_life.height as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                image.into_raw().as_ptr() as *const std::ffi::c_void,
            );
            texture
        };

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::UseProgram(shader_program.id);
            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null_mut());
            // gl::DrawArrays(gl::POINTS, 0, 4);
        }
        window.gl_swap_window();
        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
