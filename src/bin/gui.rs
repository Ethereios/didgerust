pub use makepad_widgets::*;
pub use makepad_xr::scene::*;

use cadsd_accurate::geo::Geo;
use cadsd_accurate::sim::{acoustical_simulation, get_fundamental};
use cadsd_accurate::conv::{note_name, freq_to_note};

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*;
    use mod.widgets.*;
    use mod.math.*;
    use mod.shader.*;
    use mod.draw;
    use mod.geom;

    mod.draw.DrawPhysMesh = mod.std.set_type_default() do #(DrawPhysMesh::script_shader(vm)){
        alpha_blend: false
        backface_culling: true
        vertex_pos: vertex_position(vec4f)
        fb0: fragment_output(0, vec4f)
        draw_call: uniform_buffer(draw.DrawCallUniforms)
        draw_pass: uniform_buffer(draw.DrawPassUniforms)
        draw_list: uniform_buffer(draw.DrawListUniforms)
        geom: vertex_buffer(geom.IcoVertex, geom.IcoGeom)
        u_light_dir: uniform(vec3(-0.35, 0.84, 0.42))
        u_fill_dir: uniform(vec3(0.58, 0.35, -0.62))
        v_world_clip: varying(vec4f)
        v_world: varying(vec3f)
        v_normal: varying(vec3f)

        active_camera_world_pos: fn() -> vec3f {
            let camera_world = self.draw_pass.camera_inv * vec4(0.0, 0.0, 0.0, 1.0)
            return vec3(
                camera_world.x / max(camera_world.w, 0.00001),
                camera_world.y / max(camera_world.w, 0.00001),
                camera_world.z / max(camera_world.w, 0.00001)
            )
        }

        vertex: fn() {
            let local_pos = vec3(
                self.geom.pos.x * self.scale.x,
                self.geom.pos.y * self.scale.y,
                self.geom.pos.z * self.scale.z
            )
            let local_normal = normalize(vec3(
                self.geom.normal.x / max(self.scale.x, 0.00001),
                self.geom.normal.y / max(self.scale.y, 0.00001),
                self.geom.normal.z / max(self.scale.z, 0.00001)
            ))
            let model_view = self.draw_list.view_transform * self.transform
            let world = model_view * vec4(local_pos.x, local_pos.y, local_pos.z, 1.0)
            let world_normal = normalize((model_view * vec4(local_normal.x, local_normal.y, local_normal.z, 0.0)).xyz)
            self.v_world = world.xyz
            self.v_normal = world_normal
            self.v_world_clip = vec4(world.x, world.y, world.z, 1.0)
            let view_pos = self.draw_pass.camera_view * world
            self.vertex_pos = self.draw_pass.camera_projection * view_pos
        }

        pixel: fn() {
            let normal = normalize(self.v_normal)
            let view_dir = normalize(self.active_camera_world_pos() - self.v_world)
            let key = max(dot(normal, normalize(self.u_light_dir)), 0.0)
            let fill = max(dot(normal, normalize(self.u_fill_dir)), 0.0)
            let rim = pow(max(1.0 - max(dot(normal, view_dir), 0.0), 0.0), 2.5)
            let lit = 0.18 + key * 0.72 + fill * 0.20 + rim * 0.22
            let color = self.color.xyz * lit + vec3(0.04, 0.05, 0.07) * rim
            return vec4(color, self.color.w)
        }

        fragment: fn() {
            self.fb0 = depth_clip(self.v_world_clip, self.pixel(), self.depth_clip)
        }
    }

    mod.widgets.BoreViewportBase = #(BoreViewport::register_widget(vm))
    mod.widgets.BoreViewport = set_type_default() do mod.widgets.BoreViewportBase{
        width: Fill
        height: Fill
        clear_color: #x0b1016
        draw_bg: mod.draw.DrawXrSceneTexture{}
        draw_mesh: mod.draw.DrawPhysMesh{
            backface_culling: true
        }
        camera: mod.widgets.XrCamera{
            fov_y: 45.0
            desktop_target: vec3(0.5, 0.25, 0.7)
            distance: 12.0
            distance_min: 3.0
            distance_max: 50.0
            wheel_zoom_step: 0.1
        }
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(1200, 760)
                body +: {
                    app_view := SolidView{
                        width: Fill
                        height: Fill
                        flow: Down
                        draw_bg +: {color: #x0d1116}

                        header := SolidView{
                            width: Fill
                            height: 44.0
                            flow: Right
                            align: Align{x: 0.0 y: 0.5}
                            padding: Inset{left: 14.0 right: 14.0}
                            spacing: 12.0
                            draw_bg +: {color: #x171d24}

                            title := H3{
                                text: "CADSD - Didgeridoo Analyzer"
                                draw_text +: {color: #xdfe7ee}
                            }
                            hint := Label{
                                text: "drag: orbit  wheel: zoom"
                                draw_text +: {color: #x8391a0}
                            }
                        }

                        content := View{
                            width: Fill
                            height: Fill
                            flow: Right
                            spacing: 4
                            padding: 8
                            draw_bg +: {color: #x12161d}

                            sidebar := SolidView{
                                width: 320
                                height: Fill
                                flow: Down
                                spacing: 10
                                padding: 10
                                draw_bg +: {color: #x171d24}

                                section_title := Label{
                                    width: Fill
                                    height: 24
                                    text: "Geometry"
                                    draw_text +: {color: #xdfe7ee, font_size: 14}
                                }

                                bore_style_label := Label{
                                    width: Fill
                                    height: 18
                                    text: "Bore style"
                                    draw_text +: {color: #xa0a0a0}
                                }

                                bore_style_dropdown := DropDown{
                                    width: Fill
                                    height: 28
                                    labels: ["Cone", "Cylinder", "Exponential"]
                                    selected_item: 0
                                }

                                length_label := Label{
                                    width: Fill
                                    height: 18
                                    text: "Length (mm)"
                                    draw_text +: {color: #xa0a0a0}
                                }
                                length_value := Label{
                                    width: Fill
                                    height: 18
                                    text: "950"
                                    draw_text +: {color: #xaaffaa}
                                }
                                length_slider := Slider{
                                    width: Fill
                                    height: 18
                                    min: 500.0
                                    max: 3000.0
                                    step: 10.0
                                    default: 950.0
                                }

                                top_label := Label{
                                    width: Fill
                                    height: 18
                                    text: "Top diameter (mm)"
                                    draw_text +: {color: #xa0a0a0}
                                }
                                top_value := Label{
                                    width: Fill
                                    height: 18
                                    text: "35.0"
                                    draw_text +: {color: #xaaaaff}
                                }
                                top_slider := Slider{
                                    width: Fill
                                    height: 18
                                    min: 10.0
                                    max: 50.0
                                    step: 0.5
                                    default: 35.0
                                }

                                bell_label := Label{
                                    width: Fill
                                    height: 18
                                    text: "Bell diameter (mm)"
                                    draw_text +: {color: #xa0a0a0}
                                }
                                bell_value := Label{
                                    width: Fill
                                    height: 18
                                    text: "85.0"
                                    draw_text +: {color: #xaaaaff}
                                }
                                bell_slider := Slider{
                                    width: Fill
                                    height: 18
                                    min: 20.0
                                    max: 100.0
                                    step: 0.5
                                    default: 85.0
                                }

                                segments_label := Label{
                                    width: Fill
                                    height: 18
                                    text: "Segments"
                                    draw_text +: {color: #xa0a0a0}
                                }
                                segments_value := Label{
                                    width: Fill
                                    height: 18
                                    text: "50"
                                    draw_text +: {color: #xaaaaff}
                                }
                                segments_slider := Slider{
                                    width: Fill
                                    height: 18
                                    min: 5.0
                                    max: 200.0
                                    step: 1.0
                                    default: 50.0
                                }

                                run_button := Button{
                                    width: Fill
                                    height: 36
                                    text: "Run Simulation"
                                }

                                running_label := Label{
                                    width: Fill
                                    height: 18
                                    text: "Ready"
                                    draw_text +: {color: #xffaa00}
                                }

                                fundamental_label := Label{
                                    width: Fill
                                    height: 20
                                    text: "Fundamental: -"
                                    draw_text +: {color: #x88cc88}
                                }

                                resonances_label := Label{
                                    width: Fill
                                    height: 20
                                    text: "Resonances: -"
                                    draw_text +: {color: #x88cc88}
                                }
                            }

                            viewport := mod.widgets.BoreViewport{}
                        }
                    }
                }
            }
        }
    }
}

#[derive(Script, ScriptHook, Debug)]
#[repr(C)]
pub struct DrawPhysMesh {
    #[rust(vec3(-0.35, 0.84, 0.42))]
    light_dir: Vec3f,
    #[rust(vec3(0.58, 0.35, -0.62))]
    fill_dir: Vec3f,
    #[deref]
    draw_vars: DrawVars,
    #[live]
    color: Vec4f,
    #[live]
    transform: Mat4f,
    #[live(vec3(1.0, 1.0, 1.0))]
    scale: Vec3f,
    #[live(1.0)]
    depth_clip: f32,
}

impl DrawPhysMesh {
    fn apply_uniforms(&mut self, cx: &mut CxDraw) {
        let light_dir = self.light_dir.normalize();
        let fill_dir = self.fill_dir.normalize();
        self.draw_vars.set_uniform(cx.cx, live_id!(u_light_dir), &[light_dir.x, light_dir.y, light_dir.z]);
        self.draw_vars.set_uniform(cx.cx, live_id!(u_fill_dir), &[fill_dir.x, fill_dir.y, fill_dir.z]);
    }

    fn draw(&mut self, cx: &mut CxDraw, geometry_id: GeometryId) {
        self.draw_vars.geometry_id = Some(geometry_id);
        self.apply_uniforms(cx);
        if self.draw_vars.can_instance() {
            let new_area = cx.add_instance(&self.draw_vars);
            self.draw_vars.area = cx.update_area_refs(self.draw_vars.area, new_area);
        }
    }
}

fn build_bore_geometry(segments: &[(f32, f32)]) -> (Vec<u32>, Vec<f32>) {
    let mut indices = Vec::new();
    let mut vertices = Vec::new();
    let rings = segments.len();
    for (i, &(x, d)) in segments.iter().enumerate() {
        let radius = d * 0.5;
        let y = x * 0.01;
        for j in 0..24 {
            let theta = (j as f32) * std::f32::consts::TAU / 24.0;
            let cx_ = radius * theta.cos();
            let cz = radius * theta.sin();
            vertices.extend_from_slice(&[cx_, y, cz, 1.0, theta.cos(), 0.0, theta.sin(), 0.0]);
        }
        if i + 1 < rings {
            for j in 0..24u32 {
                let next = (j + 1) % 24;
                let a = (i as u32) * 24 + j;
                let b = (i as u32) * 24 + next;
                let c = ((i + 1) as u32) * 24 + next;
                let d_ = ((i + 1) as u32) * 24 + j;
                indices.extend_from_slice(&[a, b, c, a, c, d_]);
            }
        }
    }
    (indices, vertices)
}

fn make_segments(length: f32, top: f32, bell: f32, style: u32, n: usize) -> Vec<(f32, f32)> {
    (0..n).map(|i| {
        let t = if n > 1 { i as f32 / (n - 1) as f32 } else { 0.0 };
        let x = length * t;
        let d = match style {
            0 => top + (bell - top) * t,
            1 => top,
            _ => top * (bell / top).powf(t),
        };
        (x, d)
    }).collect()
}

#[derive(Script, ScriptHook, WidgetRef, WidgetRegister)]
pub struct BoreViewport {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[live]
    draw_bg: DrawXrSceneTexture,
    #[live]
    draw_mesh: DrawPhysMesh,
    #[live(vec4(0.043, 0.063, 0.086, 1.0))]
    clear_color: Vec4f,
    #[live]
    camera: XrCamera,
    #[new]
    pass: DrawPass,
    #[new]
    draw_list: DrawList,
    #[new]
    color_texture: Texture,
    #[new]
    depth_texture: Texture,
    #[rust]
    area: Area,
    #[rust(false)]
    initialized: bool,
    #[rust]
    geometry: Option<Geometry>,
    #[rust]
    last_hash: u64,
}

impl BoreViewport {
    fn ensure_initialized(&mut self, cx: &mut Cx) {
        if self.initialized { return; }
        self.initialized = true;
        self.color_texture = Texture::new_with_format(cx, TextureFormat::RenderBGRAu8 { size: TextureSize::Auto, initial: true });
        self.depth_texture = Texture::new_with_format(cx, TextureFormat::DepthD32 { size: TextureSize::Auto, initial: true });
        self.pass.set_color_texture(cx, &self.color_texture, DrawPassClearColor::ClearWith(self.clear_color));
        self.pass.set_depth_texture(cx, &self.depth_texture, DrawPassClearDepth::ClearWith(1.0));
        cx.passes[self.pass.draw_pass_id()].keep_camera_matrix = true;

        let segs = make_segments(950.0, 35.0, 85.0, 0, 50);
        let (indices, vertices) = build_bore_geometry(&segs);
        let geometry = Geometry::new(cx);
        geometry.update(cx, indices, vertices);
        self.geometry = Some(geometry);
        self.last_hash = 0;
    }

    pub fn update_bore(&mut self, cx: &mut Cx, length: f32, top: f32, bell: f32, style: u32, n: usize) {
        let hash = (length.to_bits() as u64 ^ ((top.to_bits() as u64) << 1) ^ ((bell.to_bits() as u64) << 2) ^ ((style as u64) << 3) ^ ((n as u64) << 4))
            .wrapping_mul(0x517cc1b727265a95);
        if hash == self.last_hash { return; }
        self.last_hash = hash;

        let segs = make_segments(length, top, bell, style, n);
        let (indices, vertices) = build_bore_geometry(&segs);
        if let Some(ref mut geom) = self.geometry {
            geom.update(cx, indices, vertices);
        }
        self.area.redraw(cx);
    }

    fn draw_scene(&mut self, cx: &mut Cx3d, scene_state: SceneState3D) {
        self.draw_list.begin_always(cx);
        cx.begin_scene_3d(scene_state);
        let previous_world = cx.set_scene_world_transform_3d(Mat4f::identity());
        if let Some(ref mut geometry) = self.geometry {
            self.draw_mesh.transform = Mat4f::identity();
            self.draw_mesh.scale = vec3(1.0, 1.0, 1.0);
            self.draw_mesh.color = vec4(0.7, 0.75, 0.8, 1.0);
            self.draw_mesh.depth_clip = 0.0;
            self.draw_mesh.draw(cx, geometry.geometry_id());
        }
        if let Some(previous_world) = previous_world {
            let _ = cx.set_scene_world_transform_3d(previous_world);
        }
        cx.end_scene_3d();
        self.draw_list.end(cx);
    }
}

impl WidgetNode for BoreViewport {
    fn widget_uid(&self) -> WidgetUid { self.uid }
    fn walk(&mut self, _cx: &mut Cx) -> Walk { self.walk }
    fn area(&self) -> Area { self.area }
    fn redraw(&mut self, cx: &mut Cx) { self.area.redraw(cx); }
}

impl Widget for BoreViewport {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        self.camera.handle_desktop_interaction(cx, event);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.walk_turtle_with_area(&mut self.area, walk);
        if rect.size.x <= 1.0 || rect.size.y <= 1.0 { return DrawStep::done(); }

        self.ensure_initialized(cx.cx);
        self.camera.set_desktop_viewport_rect(rect);
        self.pass.set_size(cx, rect.size);
        self.pass.set_color_texture(cx, &self.color_texture, DrawPassClearColor::ClearWith(self.clear_color));
        self.pass.set_depth_texture(cx, &self.depth_texture, DrawPassClearDepth::ClearWith(1.0));

        cx.make_child_pass(&self.pass);
        cx.begin_pass(&self.pass, None);
        if let Some(scene_state) = self.camera.desktop_scene_state(rect, cx.time()) {
            let cx3d = &mut Cx3d::new(cx.cx);
            self.draw_scene(cx3d, scene_state);
        }
        cx.end_pass(&self.pass);

        self.draw_bg.set_scene_texture(&self.color_texture);
        self.draw_bg.draw_abs(cx, rect);
        self.area = self.draw_bg.area();
        cx.set_pass_area(&self.pass, self.area);
        DrawStep::done()
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let mut needs_viewport_update = false;
        let mut length = 950.0;
        let mut top = 35.0;
        let mut bell = 85.0;
        let mut style = 0u32;
        let mut segments = 50usize;

        if let Some(v) = self.ui.slider(cx, ids!(length_slider)).slided(actions) {
            length = v;
            self.ui.label(cx, ids!(length_value)).set_text(cx, &format!("{:.0}", v));
            needs_viewport_update = true;
        }
        if let Some(v) = self.ui.slider(cx, ids!(top_slider)).slided(actions) {
            top = v;
            self.ui.label(cx, ids!(top_value)).set_text(cx, &format!("{:.1}", v));
            needs_viewport_update = true;
        }
        if let Some(v) = self.ui.slider(cx, ids!(bell_slider)).slided(actions) {
            bell = v;
            self.ui.label(cx, ids!(bell_value)).set_text(cx, &format!("{:.1}", v));
            needs_viewport_update = true;
        }
        if let Some(v) = self.ui.slider(cx, ids!(segments_slider)).slided(actions) {
            segments = v as usize;
            self.ui.label(cx, ids!(segments_value)).set_text(cx, &format!("{}", segments));
            needs_viewport_update = true;
        }
        if let Some(v) = self.ui.drop_down(cx, ids!(bore_style_dropdown)).selected(actions) {
            style = v as u32;
            needs_viewport_update = true;
        }

        if needs_viewport_update {
            if let Some(mut vp) = self.ui.widget(cx, ids!(viewport)).borrow_mut::<BoreViewport>() {
                vp.update_bore(cx, length as f32, top as f32, bell as f32, style, segments);
            }
        }

        if self.ui.button(cx, ids!(run_button)).clicked(actions) {
            self.ui.label(cx, ids!(running_label)).set_text(cx, "Simulating...");
            std::thread::spawn(move || {
                let geo = Geo::make_cone(950.0, 35.0, 85.0, 50);
                let freqs: Vec<f64> = (20..=5000).map(|f| f as f64 * 0.1).collect();
                if let Ok(impedances) = acoustical_simulation(&geo, &freqs, "tlm_python") {
                    let (fundamental, _) = get_fundamental(&geo, "tlm_python", 20.0).unwrap_or((65.41, 1.0));
                    let note = note_name(freq_to_note(fundamental));
                    log!("Simulation done: {:.1} Hz ({})", fundamental, note);
                }
            });
        }
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        makepad_widgets::script_mod(vm);
        makepad_xr::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
