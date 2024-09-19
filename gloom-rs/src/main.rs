// Uncomment these following global attributes to silence most warnings of "low" interest:

#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]

extern crate nalgebra_glm as glm;
use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::{mem, os::raw::c_void, ptr};

mod shader;
mod util;

use glutin::event::{
   DeviceEvent,
   ElementState::{Pressed, Released},
   Event, KeyboardInput,
   VirtualKeyCode::{self, *},
   WindowEvent,
};
use glutin::event_loop::ControlFlow;

// initial window size
const INITIAL_SCREEN_W: u32 = 600;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  byte_size_of_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
   std::mem::size_of_val(val) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
   &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
   mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
   (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()

// == // Generate your VAO here
unsafe fn create_vao(vertices: &[f32], indices: &[u32], colors: &[f32]) -> u32 {
   // Generate a VAO and bind it
   let mut vao: u32 = 0;
   gl::GenVertexArrays(1, &mut vao);
   gl::BindVertexArray(vao);

   // Generate a verticy VBO and bind it
   let mut verticy_vbo: u32 = 0;
   gl::GenBuffers(1, &mut verticy_vbo);
   gl::BindBuffer(gl::ARRAY_BUFFER, verticy_vbo);

   // Fill it with data
   gl::BufferData(
      gl::ARRAY_BUFFER,
      byte_size_of_array(vertices),
      pointer_to_array(vertices),
      gl::STATIC_DRAW,
   );

   // Configure a VAP for the vertices
   let vertex_stride: i32 = 3 * size_of::<f32>();
   gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vertex_stride, ptr::null());
   gl::EnableVertexAttribArray(0);

   // Generate a color VBO and bind it
   let mut color_vbo: u32 = 0;
   gl::GenBuffers(1, &mut color_vbo);
   gl::BindBuffer(gl::ARRAY_BUFFER, color_vbo);

   // Fill it with data
   gl::BufferData(
      gl::ARRAY_BUFFER,
      byte_size_of_array(colors),
      pointer_to_array(colors),
      gl::STATIC_DRAW,
   );

   let color_stride: i32 = 4 * size_of::<f32>();
   gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, color_stride, ptr::null());
   gl::EnableVertexAttribArray(1);

   // Generate a IBO and bind it
   let mut ibo: u32 = 0;
   gl::GenBuffers(1, &mut ibo);
   gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);

   // Fill the IBO with data
   gl::BufferData(
      gl::ELEMENT_ARRAY_BUFFER,
      byte_size_of_array(indices),
      pointer_to_array(indices),
      gl::STATIC_DRAW,
   );

   // Return the id of the VAO
   vao
}

fn fill_indices(vertices: &[f32]) -> Vec<u32> {
   let mut indices: Vec<u32> = Vec::new();
   for i in 0..(vertices.len() / 3) {
      indices.push(i as u32);
   }
   indices
}

// Draw circle from the given location
// * 'location' - the centre position of the circle.
// * 'r' - the radius of the circle, minimum 0.01f.
// * 'n' - amount of vertices to define the circle, minimum 3.
fn circle_vertices(location: Vec<f32>, r: f32, n: u32) -> Vec<f32> {
   if n < 3 || r < 0.01 {
      // Return empty vertices
      return vec![];
   }

   // Calculate degrees between each verticy
   let angle: f32 = (2.0 * std::f32::consts::PI) / n as f32;

   // Calculate the x and y positions where same index belongs to eachother
   let mut vertices: Vec<f32> = Vec::new();

   for i in 0..n {
      let x: f32 = r * f32::cos(angle * i as f32) - location[0];
      let y: f32 = r * f32::sin(angle * i as f32) - location[1];
      vertices.push(x);
      vertices.push(y);
      vertices.push(0.0 + location[2]);
   }

   vertices
}

fn fill_circle_indices(vertices: &[f32]) -> Vec<u32> {
   let mut i: u32 = 1;
   let mut indices: Vec<u32> = Vec::new();

   while i < vertices.len() as u32 {
      indices.push(0);
      indices.push(i);
      indices.push(i + 1);
      i += 1;
   }

   indices
}

fn main() {
   // Set up the necessary objects to deal with windows and event handling
   let el = glutin::event_loop::EventLoop::new();
   let wb = glutin::window::WindowBuilder::new()
      .with_title("Gloom-rs")
      .with_resizable(true)
      .with_inner_size(glutin::dpi::LogicalSize::new(INITIAL_SCREEN_W, INITIAL_SCREEN_H));
   let cb = glutin::ContextBuilder::new().with_vsync(true);
   let windowed_context = cb.build_windowed(wb, &el).unwrap();
   // Acquire the OpenGL Context and load the function pointers.
   let context = unsafe { windowed_context.make_current().unwrap() };
   gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

   // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
   // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
   // windowed_context.window().set_cursor_visible(false);

   // Set up a shared vector for keeping track of currently pressed keys
   let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));

   // Set up shared tuple for tracking mouse movement between frames
   let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));

   // Set up shared tuple for tracking changes to the window size
   let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));

   // let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

   // Set up openGL
   unsafe {
      gl::Enable(gl::DEPTH_TEST);
      gl::DepthFunc(gl::LESS);
      gl::Enable(gl::CULL_FACE);
      gl::Disable(gl::MULTISAMPLE);
      gl::Enable(gl::BLEND);
      gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

      // Query OpenGL version
      let version = util::get_gl_string(gl::VERSION);
      let version_major: u32 = version.chars().next().unwrap().to_digit(10).unwrap();
      let version_minor: u32 = version.chars().nth(2).unwrap().to_digit(10).unwrap();

      // MacOS uses OpenGL 4.1, which doesn't support the debug output functionality introduced in OpenGL 4.3
      if (version_major > 4) || (version_major == 4 && version_minor >= 3) {
         gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
         gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());
      } else {
         // Simple error checking
         let error = gl::GetError();
         if error != gl::NO_ERROR {
            println!("OpenGL Error: {}", error);
         }
      }

      // Print some diagnostics
      println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
      println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
      println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
   }

   let first_frame_time = std::time::Instant::now();
   let mut previous_frame_time = first_frame_time;

   // == //
   // == // From here on down there are only internals.
   // == //

   let mut cam_pos: Vec<f32> = vec![0.0, 0.0, -5.0];
   let mut yaw: f32 = 0.0;
   let mut pitch: f32 = 0.0;

   // Start the event loop -- This is where window events are initially handled
   el.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Poll;

      match event {
         Event::WindowEvent { event: WindowEvent::Resized(physical_size), .. } => {
            println!("New window size received: {}x{}", physical_size.width, physical_size.height);
            if let Ok(mut new_size) = arc_window_size.lock() {
               *new_size = (physical_size.width, physical_size.height, true);
            }
         }
         Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
            *control_flow = ControlFlow::Exit;
         }
         // Keep track of currently pressed keys to send to the rendering thread
         Event::WindowEvent {
            event:
               WindowEvent::KeyboardInput {
                  input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. },
                  ..
               },
            ..
         } => {
            if let Ok(mut keys) = arc_pressed_keys.lock() {
               match key_state {
                  Released => {
                     if keys.contains(&keycode) {
                        let i = keys.iter().position(|&k| k == keycode).unwrap();
                        keys.remove(i);
                     }
                  }
                  Pressed => {
                     if !keys.contains(&keycode) {
                        keys.push(keycode);
                     }
                  }
               }
            }

            // Handle Escape and Q keys separately
            match keycode {
               Escape => {
                  *control_flow = ControlFlow::Exit;
               }
               _ => {}
            }
         }
         Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
            // Accumulate mouse movement
            if let Ok(mut position) = arc_mouse_delta.lock() {
               *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
            }
         }
         Event::MainEventsCleared => {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;

            // == // Set up your VAO around here

            // Vertex coordinates no UV or RGB

            let vertices: Vec<f32> = vec![
               // X, Y, Z
               // 
               // -0.90, 0.05, 0.0, 
               // -0.05, 0.90, 0.0, 
               // -0.95, 0.95, 0.0, 

               // -0.95, -0.95, 0.0, 
               // -0.05, -0.90, 0.0,
               // -0.90, -0.05, 0.0,

               // 0.05, 0.90, 0.0, 
               // 0.90, 0.05, 0.0, 
               // 0.95, 0.95, 0.0, 

               // 0.05, -0.90, 0.0, 
               // 0.95, -0.95, 0.0, 
               // 0.90, -0.05, 0.0, 

               // -0.3, -0.3, 0.0, 
               // 0.3, -0.3, 0.0, 
               // 0.0, 0.3, 0.0,

               // // Task 2
               // -0.8, 0.8, -0.9,
               // -0.9, 0.0, -0.9,
               // 0.4, -0.2, -0.9,

               // 0.8, 0.8, -0.5,
               // -0.4, -0.2, -0.5,
               // 0.9, 0.0, -0.5,

               // -0.4, -0.8, -0.1,
               // 0.4, -0.8, -0.1,
               // 0.0, 0.4, -0.1

               // Task 3
               // Front
               -0.4, 0.4, 0.4, // Top left
               -0.4, -0.4, 0.4, // Bottom left
               0.4, -0.4, 0.4, // Bottom right
               0.4, 0.4, 0.4, // Top right

               // Back
               -0.4, 0.4, -0.4, // Top left
               -0.4, -0.4, -0.4, // Bottom Left
               0.4, -0.4, -0.4, // Bottom right
               0.4, 0.4, -0.4, // Top right
            ];

            let colors: Vec<f32> = vec![
               // R, G, B, A
               // Colors for Task 1
               // 0.1, 0.2, 0.3, 1.0,
               // 0.4, 0.5, 0.6, 1.0,
               // 0.7, 0.8, 0.9, 1.0,

               // 0.9, 0.1, 0.8, 1.0,
               // 0.2, 0.7, 0.3, 1.0,
               // 0.6, 0.4, 0.5, 1.0,

               // 0.9, 0.7, 0.8, 1.0,
               // 0.6, 0.5, 0.4, 1.0,
               // 0.3, 0.2, 0.1, 1.0,

               // // Task 2
               // // 1
               // 0.1, 0.2, 0.9, 0.6,
               // 0.1, 0.2, 0.9, 0.6,
               // 0.1, 0.2, 0.9, 0.6,

               // // 2
               // 0.9, 0.3, 0.4, 0.4,
               // 0.9, 0.3, 0.4, 0.4,
               // 0.9, 0.3, 0.4, 0.4,

               // // 3
               // 0.2, 0.6, 0.3, 0.5,
               // 0.2, 0.6, 0.3, 0.5,
               // 0.2, 0.6, 0.3, 0.5,

               // Task 3
               0.9, 0.2, 0.9, 1.0,
               0.4, 0.9, 0.4, 1.0,
               0.9, 0.9, 0.2, 1.0,
               0.4, 0.9, 0.9, 1.0,

               0.3, 0.4, 0.6, 1.0,
               0.4, 0.9, 0.5, 1.0,
               0.2, 0.9, 0.2, 1.0,
               0.8, 0.9, 0.2, 1.0
            ];

            let indices: Vec<u32> = vec![
               0, 1, 2, 3, 0, 2, // Front
               0, 4, 1, 4, 5, 1, // Left
               3, 2, 6, 6, 7, 3, // Right
               6, 5, 4, 6, 4, 7, // Back
               0, 7, 4, 0, 3, 7, // Top
               6, 1, 5, 6, 2, 1, // Bottom
            ];

            let my_vao = unsafe { create_vao(&vertices, &indices, &colors) };

            // == // Set up your shaders here

            // Attaching the vertex and fragment shader to the shader builder
            let simple_shader = unsafe {
               shader::ShaderBuilder::new()
                  .attach_file("./shaders/simple.vert")
                  .attach_file("./shaders/simple.frag")
                  .link()
            };

            let name: CString = CString::new("time").unwrap();
            let time_loc: i32 = unsafe { gl::GetUniformLocation(simple_shader.program_id, name.as_ptr()) };

            let oscillating_value_name: CString = CString::new("oscVal").unwrap();
            let oscillating_loc: i32 = unsafe { gl::GetUniformLocation(simple_shader.program_id, oscillating_value_name.as_ptr()) };

            let matrix_name: CString = CString::new("matrix").unwrap();
            let matrix_loc: i32 = unsafe { gl::GetUniformLocation(simple_shader.program_id, matrix_name.as_ptr()) };

            if let Ok(mut new_size) = arc_window_size.lock() {
               if new_size.2 {
                  context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                  // window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                  new_size.2 = false;
                  println!("Window was resized to {}x{}", new_size.0, new_size.1);
                  unsafe {
                     gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32);
                  }
               }
            }

            // Handle keyboard input
            if let Ok(keys) = arc_pressed_keys.lock() {
               for key in keys.iter() {
                  match key {
                     // The `VirtualKeyCode` enum is defined here:
                     //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html
                     VirtualKeyCode::W => {
                        cam_pos[2] += 0.01;
                     }
                     VirtualKeyCode::A => {
                        yaw += 1.0;
                     }
                     VirtualKeyCode::S => {
                        cam_pos[2] -= 0.01;
                     }
                     VirtualKeyCode::D => {
                        yaw -= 1.0;
                     }
                     VirtualKeyCode::Space => {
                        cam_pos[1] -= 0.01;
                     }
                     VirtualKeyCode::LShift => {
                        cam_pos[1] += 0.01;
                     }
                     VirtualKeyCode::Q => {
                        pitch += 0.5;
                     }
                     VirtualKeyCode::E => {
                        pitch -= 0.5;
                     }
 
                     // default handler:
                     _ => {}
                  }
               }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = arc_mouse_delta.lock() {
               // == // Optionally access the accumulated mouse movement between
               // == // frames here with `delta.0` and `delta.1`

               *delta = (0.0, 0.0); // reset when done
            }

            // == // Please compute camera transforms here (exercise 2 & 3)

            // Column major matrices
            let id_mat: glm::Mat4 = glm::identity();


            // Replace m34 with -1.0 * elapsed.sin().abs() - 1.0 for the funnies.
            let trans_mat: glm::Mat4 = glm::mat4(
               1.0, 0.0, 0.0, 0.0, // First column
               0.0, 1.0, 0.0, 0.0, // Second column
               0.0, 0.0, 1.0, -4.0, // Third column
               0.0, 0.0, 0.0, 1.0, // Fourth column
            );

            // let rot_x_mat: glm::Mat4 = glm::mat4(
            //    1.0, 0.0, 0.0, 0.0,
            //    0.0, cos_x, -sin_x, 0.0,
            //    0.0, sin_x, cos_x, 0.0,
            //    0.0, 0.0, 0.0, 1.0,
            // );

            // let rot_y_mat: glm::Mat4 = glm::mat4(
            //    cos_y, 0.0, sin_y, 0.0,
            //    0.0, 1.0, 0.0, 0.0,
            //    -sin_y, 0.0, cos_y, 0.0,
            //    0.0, 0.0, 0.0, 1.0,
            // );

            // let rot_z_mat: glm::Mat4 = glm::mat4(
            //    cos_x, -sin_x, 0.0, 0.0,
            //    sin_x, cos_x, 0.0, 0.0,
            //    0.0, 0.0, 1.0, 0.0,
            //    0.0, 0.0, 0.0, 1.0,
            // );

            let trans_mat: glm::Mat4 = glm::mat4(
               1.0, 0.0, 0.0, cam_pos[0],
               0.0, 1.0, 0.0, cam_pos[1],
               0.0, 0.0, 1.0, cam_pos[2],
               0.0, 0.0, 0.0, 1.0,
            );

            let proj_mat: glm::Mat4 = glm::perspective(
               INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32,
               60.0_f32.to_radians(), 
               1.0, 
               100.0,
            );

            let rotation_x: glm::Mat4 = glm::rotation(pitch.to_radians(), &glm::vec3(1.0, 0.0, 0.0));
            let rotation_y: glm::Mat4 = glm::rotation(yaw.to_radians(), &glm::vec3(0.0, 1.0, 0.0));


            let combined_mat: glm::Mat4 = proj_mat * trans_mat * rotation_x * rotation_y * id_mat;

            unsafe {
               // Clear the color and depth buffers
               gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky
               // gl::ClearColor(1.0, 1.0, 1.0, 1.0); // night sky
               gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

               // gl::Disable(gl::CULL_FACE); //Used to see affine transformations

               // == // Issue the necessary gl:: commands to draw your scene here

               // Binding the created VAO
               gl::BindVertexArray(my_vao);

               // Activate the shader and draw the elements
               simple_shader.activate();
               unsafe { gl::Uniform1f(time_loc, elapsed) };
               unsafe { gl::Uniform1f(oscillating_loc, elapsed.sin()) };
               unsafe { gl::UniformMatrix4fv(matrix_loc, 1, gl::FALSE, combined_mat.as_ptr())}
               gl::DrawElements(gl::TRIANGLES, indices.len() as i32, gl::UNSIGNED_INT, ptr::null());
            }

            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
         }
         _ => {}
      }
   });
}
