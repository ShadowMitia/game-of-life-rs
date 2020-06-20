use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

mod shaders;

// Notes :
// - textures are upside down, probably filled the wrong way? could be flipped with opengl?

struct Texture {
    id: u32,
    width: u32,
    height: u32,
}

impl Texture {
    fn new(data: *const u8, width: u32, height: u32) -> Self {
        let texture = unsafe {
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            // set the texture wrapping parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            // set texture filtering parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB8 as i32,
                width as i32,
                height as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                data as *const std::ffi::c_void,
            );

            // gl::GenerateMipmap(gl::TEXTURE_2D);

            texture
        };

        Texture {
            id: texture,
            width,
            height,
        }
    }

    fn update(&self, data: *const u8) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                self.width as i32,
                self.height as i32,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                data as *const std::ffi::c_void,
            )
        }
    }
}

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

        for j in 0..self.height as i32 {
            for i in 0..self.width as i32 {

                // Count all neighbors for current simulation
                let mut neighbor_count = 0;

                for nj in -1 as i32..=1 {
                    for ni in -1 as i32..=1 {
                        if ni == 0 && nj == 0 {
                            continue;
                        }
                        let ni = ((i + ni) + self.width as i32) % (self.width as i32);
                        let nj = ((j + nj) + self.height as i32) % (self.height as i32);

                        let active = self.simulation[index(ni, nj, self.width as i32) as usize];
                        if *active {
                            neighbor_count += 1;
                        }
                    }
                }

                // Update with Conway Rules
                let index = index(i, j, self.width as i32);

                let is_active = self.simulation[index as usize];

                new_simulation[index as usize] = match (is_active, neighbor_count) {
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
            (3 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
        );

        gl::EnableVertexAttribArray(1);
    }

    let vertex_shader_source = r"
    #version 330 core
    layout(location = 0) in vec3 aPos;
    

    void main()
    {
        gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0f);
    }
";

    let fragment_shader_source = r"
    #version 330 core
    out vec4 FragColor;

    void main() {
        FragColor = vec4(0.8, 0.2, 0.7, 1.0);
    }
";

    let color_shader = shaders::ShaderProgram::new(vertex_shader_source, fragment_shader_source);

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

    let shader_program = shaders::ShaderProgram::new(vertex_shader_source, fragment_shader_source);

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

    // game_of_life.simulation = vec![
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &true, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &true, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &true, &true,
    //     &true, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false, &false, &false, &false, &false, &false, &false, &false,
    //     &false, &false, &false, &false,
    // ];

    let mut game_of_life_history = Vec::new();
    game_of_life_history.push(game_of_life);

    unsafe {
        gl::Viewport(0, 0, 800, 600);
    }

    let glider_width = 5;
    let glider_height = 5;
    let glider = vec![
        false, false, false, false, false, false, false, true, false, false, false, false, false,
        true, false, false, true, true, true, false, false, false, false, false, false,
    ];

    let glider2 = vec![
        false, false, false, false, false, false, true, false, false, false, false, false, true,
        true, false, false, true, true, false, false, false, false, false, false, false,
    ];

    let mut patterns = Vec::new();

    let mut g = glider;

    for _ in 0..4 {
        let mut glider_pattern = 0;
        for val in &g {
            glider_pattern <<= 1;
            glider_pattern |= if *val { 1 } else { 0 };
        }

        patterns.push(glider_pattern);

        let temp = g.clone();

        for j in 0..glider_height {
            for i in 0..glider_width {
                g[index(i, j, glider_width)] = temp[index(glider_width - 1 - j, i, glider_width)]
            }
        }
    }

    let mut g = glider2;

    for _ in 0..4 {
        let mut glider_pattern = 0;
        for val in &g {
            glider_pattern <<= 1;
            glider_pattern |= if *val { 1 } else { 0 };
        }

        patterns.push(glider_pattern);

        let temp = g.clone();

        for j in 0..glider_height {
            for i in 0..glider_width {
                g[index(i, j, glider_width)] = temp[index(glider_width - 1 - j, i, glider_width)]
            }
        }
    }

    let mut play = true;

    // Buffers

    let mut simulation_rgb: Vec<u8> =
        vec![255; (game_of_life_history[0].width * game_of_life_history[0].height * 3) as usize];

    let simulation_rgb_ptr = simulation_rgb.as_ptr();
    let texture = Texture::new(simulation_rgb_ptr, game_of_life_history[0].width, game_of_life_history[0].height);

    'running: loop {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

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
                } => play = !play,
                _ => {}
            }
        }

        // UPDATE
        if play {
            let new_game_of_life = game_of_life_history[0].simulate();
            game_of_life_history.insert(0, new_game_of_life);

            if game_of_life_history.len() > 10 {
                game_of_life_history.pop();
            }
        }

        let mut glider_board = vec![
            255;
            (game_of_life_history[0].width * game_of_life_history[0].height * 3)
                as usize
        ];

        let width = game_of_life_history[0].width;
        let height = game_of_life_history[0].height;

        let mut glider_indices = Vec::new();

        for x in 0..width {
            for y in 0..height {
                let mut pattern = 0;
                for i in 0..5 {
                    for j in 0..5 {
                        let val = game_of_life_history[0].simulation
                            [index((x + i) % width, (y + j) % height, width) as usize];
                        pattern <<= 1;
                        pattern |= if *val { 1 } else { 0 };
                    }
                }

                if patterns.iter().any(|&p| p == pattern) {
                    for i in 0..5 {
                        for j in 0..5 {
                            let index = index((x + i) % width, (y + j) % height, width) as usize;
                            let val = game_of_life_history[0].simulation[index];
                            if *val {
                                glider_indices.push(index);
                            }
                        }
                    }
                }
            }
        }

        for index in glider_indices {
            glider_board[index * 3] = 255;
            glider_board[index * 3 + 1] = 0;
            glider_board[index * 3 + 2] = 0;
        }

        // RENDERING

        simulation_rgb = simulation_rgb.iter().map(|_| 255).collect();

        let mut past = (game_of_life_history.len() - 1) as u8;
        for gol in game_of_life_history
            .iter()
            .rev()
            .take(game_of_life_history.len() - 1)
        {
            // for gol in game_of_life_history.iter().take(1) {
            for (index, &&cell) in gol.simulation.iter().enumerate() {
                if cell {
                    simulation_rgb[index * 3] = past * 12 as u8;
                    simulation_rgb[index * 3 + 1] = past * 12 as u8;
                    simulation_rgb[index * 3 + 2] = past * 12 as u8;
                }
            }
            past -= 1;
        }

        let gol = &game_of_life_history[0];

        // for gol in game_of_life_history.iter().take(1) {
        for (index, &&cell) in gol.simulation.iter().enumerate() {
            if cell {
                simulation_rgb[index * 3] = 0 as u8;
                simulation_rgb[index * 3 + 1] = 0 as u8;
                simulation_rgb[index * 3 + 2] = 0 as u8;
            }
        }

        // Creates a checkerboard!

        // for i in 0..game_of_life_history[0].width {
        //     for j in 0..game_of_life_history[0].height {
        //         let x = i * 3;
        //         let y = j * 3;
        //         let index = index(x, y, game_of_life_history[0].width) as usize;
        //         if i % 2 == 1 - (j % 2) {
        //             simulation_rgb[index] = 0;
        //             simulation_rgb[index + 1] = 0;
        //             simulation_rgb[index + 2] = 0;
        //         } else {
        //             simulation_rgb[index] = 255;
        //             simulation_rgb[index + 1] = 255;
        //             simulation_rgb[index + 2] = 255;
        //         }
        //     }
        // }

        simulation_rgb = simulation_rgb
            .iter()
            .zip(glider_board)
            .map(|(&val, col)| if val == 0 { col } else { val })
            .collect();

        let simulation_rgb_ptr = simulation_rgb.as_ptr();
        texture.update(simulation_rgb_ptr);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture.id);

            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

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
