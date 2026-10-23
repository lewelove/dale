use glow::HasContext;
use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext, Version};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use palette::Lab;
use std::num::NonZeroU32;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Window, WindowId};

const SIMPLEX_FRAG: &str = include_str!("simplex.frag");

pub struct ShaderParams {
    pub speed: f32,
    pub zoom: f32,
    pub blur: f32,
    pub edge_blur: f32,
    pub grain: f32,
    pub equalize: f32,
    pub chroma_bias: f32,
    pub mono_bias: f32,
}

pub fn run_shader_window(palette: Vec<(Lab, f32)>, params: ShaderParams) {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        palette,
        params,
        state: None,
        start_time: std::time::Instant::now(),
    };
    event_loop.run_app(&mut app).unwrap();
}

struct AppState {
    window: Window,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    gl: glow::Context,
    program: glow::Program,
    vao: glow::VertexArray,
    _vbo: glow::Buffer,
}

struct App {
    palette: Vec<(Lab, f32)>,
    params: ShaderParams,
    state: Option<AppState>,
    start_time: std::time::Instant,
}

fn hex_to_oklab(hex: &str) ->[f32; 3] {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;

    let lin = |c: f32| -> f32 {
        if c >= 0.04045 {
            ((c + 0.055) / 1.055).powf(2.4)
        } else {
            c / 12.92
        }
    };

    let lr = lin(r);
    let lg = lin(g);
    let lb = lin(b);

    let l = 0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb;
    let m = 0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb;
    let s = 0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    let ok_l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let ok_a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let ok_b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

    [ok_l, ok_a, ok_b]
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Datamile Shader Viewer")
            .with_inner_size(winit::dpi::LogicalSize::new(1000.0, 1000.0));

        let template = ConfigTemplateBuilder::new().with_alpha_size(8);
        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .unwrap();

        let window = window.unwrap();

        let raw_window_handle = window.window_handle().unwrap().as_raw();
        let gl_display = gl_config.display();
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 1))))
            .build(Some(raw_window_handle));

        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap()
        };

        let attrs = window.build_surface_attributes(Default::default()).unwrap();
        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s) as *const _)
        };

        let (program, vao, vbo) = unsafe {
            let vertex_shader_source = r#"#version 410
            in vec2 position;
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
            "#;

            let shader_version = "#version 410\n#define NUM_COLORS 24\n";
            let fragment_shader_source = format!("{}\n{}", shader_version, SIMPLEX_FRAG);

            let program = gl.create_program().expect("Cannot create program");

            let vs = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vs, vertex_shader_source);
            gl.compile_shader(vs);
            if !gl.get_shader_compile_status(vs) {
                panic!("{}", gl.get_shader_info_log(vs));
            }

            let fs = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(fs, &fragment_shader_source);
            gl.compile_shader(fs);
            if !gl.get_shader_compile_status(fs) {
                panic!("{}", gl.get_shader_info_log(fs));
            }

            gl.attach_shader(program, vs);
            gl.attach_shader(program, fs);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("{}", gl.get_program_info_log(program));
            }

            gl.delete_shader(vs);
            gl.delete_shader(fs);

            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let vertices: [f32; 12] =[
                -1.0, -1.0,
                 1.0, -1.0,
                -1.0,  1.0,
                -1.0,  1.0,
                 1.0, -1.0,
                 1.0,  1.0,
            ];
            let vertices_u8: &[u8] = std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * std::mem::size_of::<f32>(),
            );
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                vertices_u8,
                glow::STATIC_DRAW,
            );

            let pos_attrib = gl.get_attrib_location(program, "position").unwrap();
            gl.enable_vertex_attrib_array(pos_attrib);
            gl.vertex_attrib_pointer_f32(pos_attrib, 2, glow::FLOAT, false, 0, 0);

            (program, vao, vbo)
        };

        self.state = Some(AppState {
            window,
            gl_context,
            gl_surface,
            gl,
            program,
            vao,
            _vbo: vbo,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else { return };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                state.gl_surface.resize(
                    &state.gl_context,
                    NonZeroU32::new(physical_size.width.max(1)).unwrap(),
                    NonZeroU32::new(physical_size.height.max(1)).unwrap(),
                );
                unsafe {
                    state.gl.viewport(0, 0, physical_size.width as i32, physical_size.height as i32);
                }
            }
            WindowEvent::RedrawRequested => {
                unsafe {
                    state.gl.clear_color(0.0, 0.0, 0.0, 1.0);
                    state.gl.clear(glow::COLOR_BUFFER_BIT);

                    state.gl.use_program(Some(state.program));

                    let time = self.start_time.elapsed().as_secs_f32();
                    if let Some(loc) = state.gl.get_uniform_location(state.program, "iTime") {
                        state.gl.uniform_1_f32(Some(&loc), time);
                    }
                    if let Some(loc) = state.gl.get_uniform_location(state.program, "iRandom") {
                        state.gl.uniform_1_f32(Some(&loc), 0.0);
                    }
                    if let Some(loc) = state.gl.get_uniform_location(state.program, "iSpeed") {
                        state.gl.uniform_1_f32(Some(&loc), self.params.speed);
                    }
                    if let Some(loc) = state.gl.get_uniform_location(state.program, "iZoom") {
                        state.gl.uniform_1_f32(Some(&loc), self.params.zoom);
                    }
                    if let Some(loc) = state.gl.get_uniform_location(state.program, "iBlur") {
                        state.gl.uniform_1_f32(Some(&loc), self.params.blur);
                    }
                    
                    let count = self.palette.len().min(24);
                    
                    if let Some(loc) = state.gl.get_uniform_location(state.program, "iEdgeBlur") {
                        let edge_blur_factor = if count > 0 { count as f32 / 24.0 } else { 1.0 };
                        state.gl.uniform_1_f32(Some(&loc), self.params.edge_blur * edge_blur_factor);
                    }
                    if let Some(loc) = state.gl.get_uniform_location(state.program, "iGrain") {
                        state.gl.uniform_1_f32(Some(&loc), self.params.grain);
                    }
                    
                    let size = state.window.inner_size();
                    if let Some(loc) = state.gl.get_uniform_location(state.program, "iResolution") {
                        state.gl.uniform_2_f32(Some(&loc), size.width as f32, size.height as f32);
                    }

                    if let Some(loc) = state.gl.get_uniform_location(state.program, "iCount") {
                        state.gl.uniform_1_i32(Some(&loc), count as i32);
                    }

                    let mut colors =[0.0f32; 24 * 3]; 
                    let mut ratios =[0.0f32; 24];
                    let avg_ratio = if count > 0 { 1.0 / count as f32 } else { 1.0 };

                    let mut total_raw = 0.0;
                    let mut raw_ratios = vec![0.0; count];
                    for i in 0..count {
                        raw_ratios[i] = self.palette[i].1;
                        total_raw += raw_ratios[i];
                    }
                    
                    if total_raw > 0.0 {
                        for i in 0..count {
                            raw_ratios[i] /= total_raw;
                        }
                    } else {
                        for i in 0..count {
                            raw_ratios[i] = avg_ratio;
                        }
                    }
                    
                    for i in 0..count {
                        let lab = self.palette[i].0;
                        let hex = crate::kmeans::lab_to_hex(lab);
                        let oklab = hex_to_oklab(&hex);

                        colors[i * 3 + 0] = oklab[0];
                        colors[i * 3 + 1] = oklab[1];
                        colors[i * 3 + 2] = oklab[2];
                        
                        let mut current_ratio = (raw_ratios[i] * (1.0 - self.params.equalize)) + (avg_ratio * self.params.equalize);

                        let chroma = (lab.a.powi(2) + lab.b.powi(2)).sqrt();
                        let normalized_chroma = (chroma / 128.0).clamp(0.0, 1.0);

                        if self.params.chroma_bias > 0.0 {
                            current_ratio *= 1.0 + (normalized_chroma * self.params.chroma_bias);
                        }

                        if self.params.mono_bias > 0.0 {
                            let mono_factor = (1.0 - normalized_chroma).max(0.0);
                            current_ratio *= 1.0 + (mono_factor * self.params.mono_bias);
                        }

                        ratios[i] = current_ratio;
                    }

                    if self.params.chroma_bias > 0.0 || self.params.mono_bias > 0.0 {
                        let total_biased: f32 = ratios.iter().take(count).sum();
                        if total_biased > 0.0 {
                            for i in 0..count {
                                ratios[i] /= total_biased;
                            }
                        }
                    }

                    if let Some(loc) = state.gl.get_uniform_location(state.program, "iColorsOklab") {
                        state.gl.uniform_3_f32_slice(Some(&loc), &colors);
                    }
                    if let Some(loc) = state.gl.get_uniform_location(state.program, "iRatios") {
                        state.gl.uniform_1_f32_slice(Some(&loc), &ratios);
                    }

                    state.gl.bind_vertex_array(Some(state.vao));
                    state.gl.draw_arrays(glow::TRIANGLES, 0, 6);
                }
                state.gl_surface.swap_buffers(&state.gl_context).unwrap();
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}
