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
mod mesh;
mod scene_graph;

use gl::types::GLenum;
use glm::Vec3;
use glutin::event::{
   DeviceEvent,
   ElementState::{Pressed, Released},
   Event, KeyboardInput,
   VirtualKeyCode::{self, *},
   WindowEvent,
};
use glutin::event_loop::ControlFlow;
use mesh::{Helicopter, Mesh};
use scene_graph::{Node, SceneNode};

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

unsafe fn gen_vbo_buffer<T>(array_data: &[T], target: GLenum, usage: GLenum) {
   // Generate a vbo
   let mut data_vbo: u32 = 0;
   gl::GenBuffers(1, &mut data_vbo);
   gl::BindBuffer(target, data_vbo);

   gl::BufferData(
      target,
      byte_size_of_array(array_data),
      pointer_to_array(array_data),
      usage
   );
}


// == // Generate your VAO here
unsafe fn create_vao(vertices: &[f32], indices: &[u32], colors: &[f32], normals: &[f32]) -> u32 {
   // Generate a VAO and bind it
   let mut vao: u32 = 0;
   gl::GenVertexArrays(1, &mut vao);
   gl::BindVertexArray(vao);

   // Generate a verticy VBO and bind it
   gen_vbo_buffer(vertices, gl::ARRAY_BUFFER, gl::STATIC_DRAW);

   // Configure a VAP for the vertices
   let vertex_stride: i32 = 3 * size_of::<f32>();
   gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, vertex_stride, ptr::null());
   gl::EnableVertexAttribArray(0);


   // Generate a normal VBO and bind it
   gen_vbo_buffer(normals, gl::ARRAY_BUFFER, gl::STATIC_DRAW);

   // Configure a VAP for the normals
   let normal_stride: i32 = 3 * size_of::<f32>();
   gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, normal_stride, ptr::null());
   gl::EnableVertexAttribArray(1);


   // Generate a color VBO and bind it
   gen_vbo_buffer(colors, gl::ARRAY_BUFFER, gl::STATIC_DRAW);

   // Color VAP
   let color_stride: i32 = 4 * size_of::<f32>();
   gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, color_stride, ptr::null());
   gl::EnableVertexAttribArray(2);

   // Generate a IBO and bind it
   gen_vbo_buffer(indices, gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);

   // Return the id of the VAO
   gl::BindVertexArray(0);
   vao
}

fn setup() {
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

   let first_frame_time = std::time::Instant::now();
   let mut previous_frame_time = first_frame_time;

   setup();

   // == //
   // == // From here on down there are only internals.
   // == //

   // Camera Positions
   let mut cam_pos: Vec3 = glm::vec3(0.0, 0.0, 3.0);
   let mut cam_dir: Vec3 = -cam_pos.normalize();
   let mut up_vec: Vec3 = glm::vec3(0.0, 1.0, 0.0);
   let mut yaw: f32 = -90.0;
   let mut pitch: f32 = 0.0;
   let speed: f32 = 0.1;
   let sens: f32 = 0.25;

   let lunar_terrain_mesh: Mesh  = mesh::Terrain::load("resources/lunarsurface.obj");
   let helicopter_mesh: Helicopter = mesh::Helicopter::load("resources/helicopter.obj");

   // == // Set up your VAO around here
   // Old vertices and indices for the 3D cube lies in restCode.txt for cleanup.

   let lunar_terrain_vao = unsafe { 
      create_vao(
         &lunar_terrain_mesh.vertices, 
         &lunar_terrain_mesh.indices,
         &lunar_terrain_mesh.colors,
         &lunar_terrain_mesh.normals
      )
   };

   let helicopter_vaos: Vec<u32> = unsafe {
      vec![
         create_vao(&helicopter_mesh.body.vertices, &helicopter_mesh.body.indices, &helicopter_mesh.body.colors, &helicopter_mesh.body.normals),
         create_vao(&helicopter_mesh.door.vertices, &helicopter_mesh.door.indices, &helicopter_mesh.door.colors, &helicopter_mesh.door.normals),
         create_vao(&helicopter_mesh.main_rotor.vertices, &helicopter_mesh.main_rotor.indices, &helicopter_mesh.main_rotor.colors, &helicopter_mesh.main_rotor.normals),
         create_vao(&helicopter_mesh.tail_rotor.vertices, &helicopter_mesh.tail_rotor.indices, &helicopter_mesh.tail_rotor.colors, &helicopter_mesh.tail_rotor.normals)
      ]
   };

   let helicopter_indices: Vec<i32> = vec![
      helicopter_mesh.body.index_count,
      helicopter_mesh.door.index_count,
      helicopter_mesh.main_rotor.index_count,
      helicopter_mesh.tail_rotor.index_count
   ];

   let mut terrain_scene_node: Node = SceneNode::from_vao(lunar_terrain_vao, lunar_terrain_mesh.index_count);
   let mut heli_body_scene_node: Node = SceneNode::from_vao(helicopter_vaos[0], helicopter_indices[0]);
   let mut heli_door_scene_node: Node = SceneNode::from_vao(helicopter_vaos[1], helicopter_indices[1]);
   let mut heli_main_rotor_scene_node: Node = SceneNode::from_vao(helicopter_vaos[2], helicopter_indices[2]);
   let mut heli_tail_rotor_scene_node: Node = SceneNode::from_vao(helicopter_vaos[3], helicopter_indices[3]);

   let mut scene: Node = SceneNode::new(); 

   heli_body_scene_node.add_child(&heli_door_scene_node);
   heli_body_scene_node.add_child(&heli_main_rotor_scene_node);
   heli_body_scene_node.add_child(&heli_tail_rotor_scene_node);
   terrain_scene_node.add_child(&heli_body_scene_node);
   scene.add_child(&terrain_scene_node);

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

            // Define the normalized direction.
            // We only want the direction on the xz plane
            cam_dir.x = glm::cos(&glm::radians(&glm::vec1(yaw))).x;
            cam_dir.z = glm::sin(&glm::radians(&glm::vec1(yaw))).x;
            cam_dir = cam_dir.normalize();
            let cam_dir_left4: glm::Vec4 = glm::rotation((90.0 as f32).to_radians(), &glm::vec3(0.0, 1.0, 0.0)) * 
                                           glm::vec4(cam_dir.x, cam_dir.y, cam_dir.z, 1.0);
            let mut cam_dir_left: glm::Vec3 = glm::vec3(cam_dir_left4.x, cam_dir_left4.y, cam_dir_left4.z);
            cam_dir_left = cam_dir_left.normalize();


            // Handle keyboard input
            if let Ok(keys) = arc_pressed_keys.lock() {
               for key in keys.iter() {
                  match key {
                     // The `VirtualKeyCode` enum is defined here:
                     //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html

                     // New position = (pos + dir) * speed = pos
                     VirtualKeyCode::W => {
                        cam_pos.x += cam_dir.x * speed;
                        cam_pos.z += cam_dir.z * speed;
                     }
                     VirtualKeyCode::A => {
                        cam_pos.x += cam_dir_left.x * speed;
                        cam_pos.z += cam_dir_left.z * speed;
                     }
                     VirtualKeyCode::S => {
                        cam_pos.x -= cam_dir.x * speed;
                        cam_pos.z -= cam_dir.z * speed;
                     }
                     VirtualKeyCode::D => {
                        cam_pos.x += -cam_dir_left.x * speed;
                        cam_pos.z += -cam_dir_left.z * speed;
                     }
                     VirtualKeyCode::Space => {
                        cam_pos.y += speed;
                     }
                     VirtualKeyCode::LShift => {
                        cam_pos.y -= speed;
                     }
                     VirtualKeyCode::Q => {
                        yaw -= sens;
                     }
                     VirtualKeyCode::E => {
                        yaw += sens;
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

            // Using a look at matrix, which uses the properties: Up vector and direction vector.
            let combined_pos_dir: glm::Vec3 = cam_pos + cam_dir;
            let view: glm::Mat4 = glm::look_at(&cam_pos, &combined_pos_dir, &up_vec);
            let rotation_x: glm::Mat4 = glm::rotation(pitch.to_radians(), &glm::vec3(1.0, 0.0, 0.0));

            let proj_mat: glm::Mat4 = glm::perspective(
               INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32,
               60.0_f32.to_radians(), 
               1.0, 
               1000.0,
            );
            // New position = (pos + dir) * speed = pos

            let combined_mat: glm::Mat4 = proj_mat * rotation_x * view;


            unsafe {
               // Clear the color and depth buffers
               gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky
               // gl::ClearColor(1.0, 1.0, 1.0, 1.0); // night sky
               gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

               // gl::Disable(gl::CULL_FACE); //Used to see affine transformations

               // == // Issue the necessary gl:: commands to draw your scene here

               // Activate the shader and draw the elements
               simple_shader.activate();
               unsafe { gl::Uniform1f(time_loc, elapsed) };
               unsafe { gl::Uniform1f(oscillating_loc, elapsed.sin()) };
               unsafe { gl::UniformMatrix4fv(matrix_loc, 1, gl::FALSE, combined_mat.as_ptr()) }

               // Binding the created VAO
               gl::BindVertexArray(lunar_terrain_vao);
               gl::DrawElements(gl::TRIANGLES,  lunar_terrain_mesh.index_count, gl::UNSIGNED_INT, ptr::null());

               for vao in 0..helicopter_vaos.len() {
                  gl::BindVertexArray(helicopter_vaos[vao]);
                  gl::DrawElements(gl::TRIANGLES,  helicopter_indices[vao], gl::UNSIGNED_INT, ptr::null());
               }
            }

            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
         }
         _ => {}
      }
   });
}
