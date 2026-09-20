pub use makepad_widgets::chart::DataPoint;
pub use makepad_widgets::*;
pub use makepad_xr::scene::*;

use cadsd_accurate::conv::{freq_to_note, note_name};
use cadsd_accurate::evo::{GeoGenome, LossFunctionType, Nuevolution};
use cadsd_accurate::geo::Geo;
use cadsd_accurate::loss::TairuaLoss;
use cadsd_accurate::sim::{acoustical_simulation, get_log_simulation_frequencies};
use makepad_render::scene::set_pass_camera;
use open;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use rand;

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
        geom: vertex_buffer(geom.PbrVertex, geom.PbrGeom)
        u_light_dir: uniform(vec3(-0.35, 0.84, 0.42))
        u_fill_dir: uniform(vec3(0.58, 0.35, -0.62))
        u_wireframe: uniform(float(0.0))
        v_world_clip: varying(vec4f)
        v_world: varying(vec3f)
        v_normal: varying(vec3f)
        v_uv: varying(vec2f)

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
                self.geom.pos_nx.x,
                self.geom.pos_nx.y,
                self.geom.pos_nx.z
            )
            let local_normal = normalize(vec3(
                self.geom.pos_nx.w,
                self.geom.ny_nz_uv.x,
                self.geom.ny_nz_uv.y
            ))
            let local_uv = vec2(self.geom.ny_nz_uv.z, self.geom.ny_nz_uv.w)
            let model_view = self.draw_list.view_transform * self.transform
            let world = model_view * vec4(local_pos.x, local_pos.y, local_pos.z, 1.0)
            let world_normal = normalize((model_view * vec4(local_normal.x, local_normal.y, local_normal.z, 0.0)).xyz)
            self.v_world = world.xyz
            self.v_normal = world_normal
            self.v_uv = local_uv
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
            if self.u_wireframe > 0.5 {
                let bary = vec3(self.v_uv.x, self.v_uv.y, 1.0 - self.v_uv.x - self.v_uv.y)
                let fw = vec3(
                    abs(dFdx(bary.x)) + abs(dFdy(bary.x)),
                    abs(dFdx(bary.y)) + abs(dFdy(bary.y)),
                    abs(dFdx(bary.z)) + abs(dFdy(bary.z))
                )
                let edge = min(min(bary.x / max(fw.x, 0.00001), bary.y / max(fw.y, 0.00001)), bary.z / max(fw.z, 0.00001))
                let line = 1.0 - clamp(edge - 0.6, 0.0, 1.0)
                let edge_color = vec3(0.2, 0.7, 0.9)
                let wire = mix(color, edge_color, line * 0.9)
                return vec4(wire.x, wire.y, wire.z, 1.0)
            }
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
            backface_culling: false
        }
        camera: mod.widgets.XrCamera{
            fov_y: 45.0
            desktop_target: vec3(0.0, 47.5, 0.0)
            distance: 70.0
            distance_min: 10.0
            distance_max: 150.0
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
                              new_batch: true

                             title := H3{
                                 text: "CADSD - Didgeridoo Analyzer"
                                 draw_text +: {color: #dfe7ee}
                             }
                             hint := Label{
                                  text: "drag: orbit  wheel: zoom"
                                  draw_text +: {color: #x8391a0}
                              }

                             running_label := Label{
                                     width: 100
                                     height: 24
                                     text: "Ready"
                                     draw_text +: {color: #x88cc88}
                             }

                             current_view_selector := DropDown{
                                     width: 100
                                     height: 24
                                     labels: ["Setup", "Segments", "Bubbles", "Optimization", "Export"]
                                     selected_item: 0
                                     draw_bg +: {color: #x202020}
                                     draw_text +: {color: #dfe7ee, font_size: 12}
                                 }
                         }
 menu_bar := View{
                                  width: Fill
                                  height: Fit
                                  flow: Right
                                  spacing: 2
                                  align: Align{x: 0.0 y: 0.5}

                                  file_menu := DropDown{
                                      width: Fit
                                      height: 24
                                      labels: ["New Project", "Open Project", "Save Project", "Save As...", "Export", "Quit"]
                                      selected_item: 0
                                      draw_text +: {color: #dfe7ee, font_size: 12}
                                  }

                                  edit_menu := DropDown{
                                      width: Fit
                                      height: 24
                                      labels: ["Undo", "Redo", "Preferences"]
                                      selected_item: 0
                                      draw_text +: {color: #dfe7ee, font_size: 12}
                                  }

                                  view_menu := DropDown{
                                      width: Fit
                                      height: 24
                                      labels: ["Toggle Sidebar", "Toggle Wireframe", "Toggle Cross-Section", "Zoom Extents", "Full Screen"]
                                      selected_item: 0
                                      draw_text +: {color: #dfe7ee, font_size: 12}
                                  }

                                  theme_menu := DropDown{
                                      width: Fit
                                      height: 24
                                      labels: ["Dark Mode", "Light Mode", "Auto"]
                                      selected_item: 0
                                      draw_text +: {color: #dfe7ee, font_size: 12}
                                  }

                                  help_menu := DropDown{
                                      width: Fit
                                      height: 24
                                      labels: ["Documentation", "About"]
                                      selected_item: 0
                                      draw_text +: {color: #dfe7ee, font_size: 12}
                                  }
                              }

content := View{
                            width: Fill
                            height: Fill
                            flow: Right
                            spacing: 4
                            padding: 8
                            draw_bg +: {color: #x12161d}
                            show_bg: true
                            new_batch: true

                            sidebar := SolidView{
                                   width: 320
                                   height: Fill
                                   flow: Down
                                   spacing: 10
                                   padding: 10
                                   draw_bg +: {color: #x171d24}
                                   new_batch: true

scroller := ScrollYView{
                                       width: Fill
                                       height: Fill
                                       flow: Down
                                       spacing: 10
                                       padding: 0

                                       section_title := Label{
                                          width: Fill
                                          height: 40
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
                                            labels: ["Cone", "Kigali", "Mbeya"]
                                            selected_item: 0
                                        }

                                      length_label := Label{
                                          width: Fill
                                          height: 18
                                          text: "Length (mm)"
                                          draw_text +: {color: #xa0a0a0}
                                      }

length_value := Label{
                                           width: 80
                                           height: 18
                                           text: "950"
                                           draw_text +: {color: #xaaaaff, font_size: 14}
                                       }

                                      length_slider := Slider{
                                          width: Fill
                                          height: 40
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
                                           width: 80
                                           height: 18
                                           text: "35.0"
                                           draw_text +: {color: #xaaaaff, font_size: 14}
                                       }

                                      top_slider := Slider{
                                          width: Fill
                                          height: 40
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
                                           width: 80
                                           height: 18
                                           text: "85.0"
                                           draw_text +: {color: #aaaaff, font_size: 14}
                                       }

                                      bell_slider := Slider{
                                          width: Fill
                                          height: 40
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
                                           width: 80
                                           height: 18
                                           text: "50"
                                           draw_text +: {color: #xaaaaff, font_size: 14}
                                       }

                                      segments_slider := Slider{
                                          width: Fill
                                          height: 40
                                          min: 5.0
                                          max: 200.0
                                          step: 1.0
                                          default: 50.0
                                      }

                                      bore_curve_label := Label{
                                          width: Fill
                                          height: 18
                                          text: "Bore curve"
                                          draw_text +: {color: #xa0a0a0}
                                      }

bore_curve_value := Label{
                                            width: 80
                                            height: 18
                                            text: "1.0"
                                            draw_text +: {color: #xaaaaff, font_size: 14}
                                        }

bore_curve_slider := Slider{
                                            width: Fill
                                            height: 40
                                            min: -2.0
                                            max: 2.0
                                            step: 0.1
                                            default: 1.0
                                        }

                                       geo_profile := Label{
                                           width: Fill
                                           height: 18
                                           text: "Profile: Cone"
                                           draw_text +: {color: #x88cc88}
                                       }

bubble_section := View{
                                            width: Fill
                                            height: Fit
                                            flow: Down
                                            spacing: 4
                                            padding: 4
                                            draw_bg +: {color: #x2d2d2d}
                                            show_bg: true
                                            new_batch: true

bubble_title := Label{
                                                width: Fill
                                                height: 18
                                                text: "Bubbles (pos, width, height) mm"
                                                draw_text +: {color: #xdfe7ee}
                                            }

                                           bubble_add_row := View{
                                               width: Fill
                                               height: 20
                                               flow: Right
                                               spacing: 4

                                               bubble_pos_input := TextInput{
                                                   width: 60
                                                   height: 18
                                                   text: "400"
                                                   draw_text +: {color: #xaaaaff, font_size: 12}
                                               }
                                               bubble_width_input := TextInput{
                                                   width: 60
                                                   height: 18
                                                   text: "20"
                                                   draw_text +: {color: #xaaaaff, font_size: 12}
                                               }
                                               bubble_height_input := TextInput{
                                                   width: 60
                                                   height: 18
                                                   text: "5"
                                                   draw_text +: {color: #xaaaaff, font_size: 12}
                                               }
                                               bubble_add_button := Button{
                                                   width: 60
                                                   height: 24
                                                   text: "Add Bubble"
                                               }
                                               bubble_remove_button := Button{
                                                   width: 60
                                                   height: 24
                                                   text: "Remove"
                                               }
                                           }

                                           bubble_count := Label{
                                               width: Fill
                                               height: 18
                                               text: "Bubbles: 0"
                                               draw_text +: {color: #x88cc88}
                                           }
                                       }

segment_ops_section := View{
                                            width: Fill
                                            height: Fit
                                            flow: Down
                                            spacing: 4
                                            padding: 4
                                            draw_bg +: {color: #x2d2d2d}
                                            show_bg: true
                                            new_batch: true

                                            seg_ops_title := Label{
                                                width: Fill
                                                height: 18
                                                text: "Segment Ops (move, sort)"
                                                draw_text +: {color: #xdfe7ee}
                                            }

                                            seg_ops_row := View{
                                                width: Fill
                                                height: 20
                                                flow: Right
                                                spacing: 4

                                               seg_start_input := TextInput{
                                                   width: 50
                                                   height: 18
                                                   text: "0"
                                                   draw_text +: {color: #xaaaaff, font_size: 12}
                                               }
                                               seg_end_input := TextInput{
                                                   width: 50
                                                   height: 18
                                                   text: "0"
                                                   draw_text +: {color: #xaaaaff, font_size: 12}
                                               }
                                               seg_offset_input := TextInput{
                                                   width: 60
                                                   height: 18
                                                   text: "0"
                                                   draw_text +: {color: #xaaaaff, font_size: 12}
                                               }
                                                seg_move_button := Button{
                                                    width: 60
                                                    height: 24
                                                    text: "Move"
                                                }
                                                seg_sort_button := Button{
                                                    width: 60
                                                    height: 24
                                                    text: "Sort"
        }
    }

}

                                        // Segment Editor toggle and list
                                        segment_editor_toggle := Button{
                                            width: Fill
                                            height: 24
                                            text: "Show Segment Editor"
                                        }

                                        segment_editor_list := Label{
                                            width: Fill
                                            height: Fit
                                            text: "Segments: 0"
                                            draw_text +: {color: #x88cc88, font_size: 12}
                                        }

mouthpiece_section := View{
                                             width: Fill
                                             height: Fit
                                             flow: Down
                                             spacing: 4
                                             padding: 4
                                             draw_bg +: {color: #x2d2d2d}
                                             show_bg: true
                                             new_batch: true

                                            mp_title := Label{
                                                width: Fill
                                                height: 18
                                                text: "Mouthpiece"
                                                draw_text +: {color: #xdfe7ee}
                                            }

                                            mp_toggle := Button{
                                                width: Fill
                                                height: 24
                                                text: "Enable Mouthpiece"
                                            }

                                            mp_type_dropdown := DropDown{
                                                width: Fill
                                                height: 28
                                                labels: ["None", "Reed", "Embouchure", "Fipple", "Cup"]
                                                selected_item: 0
                                            }

                                            mp_length_label := Label{
                                                width: Fill
                                                height: 18
                                                text: "Length (mm)"
                                                draw_text +: {color: #xa0a0a0}
                                            }

                                            mp_length_slider := Slider{
                                                width: Fill
                                                height: 40
                                                min: 10.0
                                                max: 200.0
                                                step: 5.0
                                                default: 0.0
                                            }

                                            mp_length_value := Label{
                                                width: Fill
                                                height: 18
                                                text: "Length: 0 mm"
                                                draw_text +: {color: #xdfe7ee}
                                            }

                                            mp_diameter_label := Label{
                                                width: Fill
                                                height: 18
                                                text: "Diameter (mm)"
                                                draw_text +: {color: #xa0a0a0}
                                            }

                                            mp_diameter_slider := Slider{
                                                width: Fill
                                                height: 40
                                                min: 10.0
                                                max: 100.0
                                                step: 5.0
                                                default: 0.0
                                            }

                                            mp_diameter_value := Label{
                                                width: Fill
                                                height: 18
                                                text: "Diameter: 0 mm"
                                                draw_text +: {color: #xdfe7ee}
                                            }
                                        }

                                        holes_section := View{
                                            width: Fill
                                            height: Fit
                                            flow: Down
                                            spacing: 4
                                            padding: 4
                                            draw_bg +: {color: #x2d2d2d}
                                            show_bg: true
                                            new_batch: true

holes_title := Label{
                                                    width: Fill
                                                    height: 18
                                                    text: "Finger Holes (Experimental)"
                                                    draw_text +: {color: #xdfe7ee}
                                                }

                                            holes_toggle := Button{
                                                width: Fill
                                                height: 24
                                                text: "Enable Holes"
                                            }

                                            holes_count_label := Label{
                                                width: Fill
                                                height: 18
                                                text: "Hole count: 0"
                                                draw_text +: {color: #xdfe7ee}
                                            }

                                            holes_count_slider := Slider{
                                                width: Fill
                                                height: 40
                                                min: 0.0
                                                max: 12.0
                                                step: 1.0
                                                default: 0.0
                                            }

                                            holes_list := Label{
                                                width: Fill
                                                height: Fit
                                                text: "Holes: none"
                                                draw_text +: {color: #xb0b0b0, font_size: 11}
                                            }
                                        }

loss_section := View{
                                             width: Fill
                                             height: Fit
                                             flow: Down
                                             spacing: 4
                                             padding: 4
                                             draw_bg +: {color: #x2d2d2d}
                                             show_bg: true
                                             new_batch: true

                                            loss_title := Label{
                                                width: Fill
                                                height: 18
                                                text: "Loss Breakdown"
                                                draw_text +: {color: #xdfe7ee}
                                            }

                                            loss_total := Label{
                                                width: Fill
                                                height: 18
                                                draw_text +: {color: #x88cc88, font_size: 13}
                                            }

                                            loss_w_fund := Label{
                                                width: Fill
                                                height: 18
                                                draw_text +: {color: #xdfe7ee}
                                            }

                                            loss_fund_slider := Slider{
                                                width: Fill
                                                height: 30
                                                min: 0.0
                                                max: 10.0
                                                step: 0.5
                                                default: 5.0
                                            }

                                            loss_w_harm := Label{
                                                width: Fill
                                                height: 18
                                                draw_text +: {color: #xdfe7ee}
                                            }

                                            loss_harm_slider := Slider{
                                                width: Fill
                                                height: 30
                                                min: 0.0
                                                max: 10.0
                                                step: 0.5
                                                default: 5.0
                                            }

                                            loss_w_peak := Label{
                                                width: Fill
                                                height: 18
                                                draw_text +: {color: #xdfe7ee}
                                            }

                                            loss_peak_slider := Slider{
                                                width: Fill
                                                height: 30
                                                min: 0.0
                                                max: 10.0
                                                step: 0.5
                                                default: 5.0
                                            }

                                            loss_chart := mod.widgets.BarChart{
                                                width: Fill
                                                height: 120
                                                draw_bg +: {color: #x12161d}
                                            }

                                            loss_target_label := Label{
                                                width: Fill
                                                height: 18
                                                text: "Target Freq (Hz)"
                                                draw_text +: {color: #xa0a0a0}
                                            }

                                            loss_target_input := TextInput{
                                                width: Fill
                                                height: 24
                                                text: "440"
                                                draw_text +: {color: #xaaaaff, font_size: 12}
                                            }
                                        }

                                        export_section := View{
                                            width: Fill
                                            height: Fit
                                            flow: Right
                                            spacing: 4
                                            padding: 4
                                            draw_bg +: {color: #x2d2d2d}
                                            show_bg: true
                                            new_batch: true

                                            export_csv_button := Button{
                                                width: Fill
                                                height: 28
                                                text: "Export CSV"
                                            }
                                            export_json_button := Button{
                                                width: Fill
                                                height: 28
                                                text: "Export JSON"
                                            }
                                        }

                                        optimization_section := View{
                                            width: Fill
                                            height: Fit
                                            flow: Down
                                            spacing: 2
                                            padding: 4
                                            draw_bg +: {color: #x121212}
                                            show_bg: true
                                            new_batch: true

                                            opt_title := Label{
                                                width: Fill
                                                height: 18
                                                text: "Optimization"
                                                draw_text +: {color: #xffcc66}
                                            }

                                            opt_target_label := Label{
                                                width: Fill
                                                height: 16
                                                text: "Target Freq: - Hz"
                                                draw_text +: {color: #xdfe7ee}
                                            }

                                            opt_progress_label := Label{
                                                width: Fill
                                                height: 16
                                                text: "Status: idle"
                                                draw_text +: {color: #xb0b0b0}
                                            }

                                            opt_buttons := View{
                                                width: Fill
                                                height: Fit
                                                flow: Right
                                                spacing: 4

                                                opt_start_button := Button{
                                                    width: Fill
                                                    height: 24
                                                    text: "Start Opt"
                                                }

                                                opt_stop_button := Button{
                                                    width: Fill
                                                    height: 24
                                                    text: "Stop"
                                                }

                                                opt_settings_button := Button{
                                                    width: Fill
                                                    height: 24
                                                    text: "Evolution Settings"
                                                }
                                            }

                                            generational_loss_chart := LineChart{
                                                width: Fill
                                                height: 200
                                                draw_bg +: {color: #x0d1116}
                                                grid: true
                                                title: "Loss Over Generations"
                                            }
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
                               }

main_area := View{
                                 width: Fill
                                 height: Fill
                                 flow: Down
                                 spacing: 4
                                 padding: 0
                                 draw_bg +: {color: #x12161d}
                                 show_bg: true
                                 new_batch: true

                                viewport := mod.widgets.BoreViewport{
                                    height: 400
                                }

impedance_preview := View{
                                        width: Fill
                                        height: 300
                                        flow: Down
                                        spacing: 4
                                        padding: 8
                                        draw_bg +: {color: #x171d24}
                                        show_bg: true
                                        new_batch: true

                                        impedance_title := Label{
                                            width: Fill
                                            height: 20
                                            text: "Impedance Spectrum"
                                            draw_text +: {color: #xdfe7ee}
                                        }

                                        impedance_chart := mod.widgets.LineChart{
                                            width: Fill
                                            height: Fill
                                            draw_bg +: {color: #x12161d}
                                        }
                                    }

                                    cross_section := View{
                                        width: Fill
                                        height: Fit
                                        flow: Down
                                        spacing: 4
                                        padding: 8
                                        draw_bg +: {color: #x171d24}
                                        show_bg: true
                                        new_batch: true

                                        cross_section_title := Label{
                                            width: Fill
                                            height: 20
                                            text: "Cross-Section View"
                                            draw_text +: {color: #xdfe7ee}
                                        }

                                        cross_section_plot := mod.widgets.LineChart{
                                            width: Fill
                                            height: Fill
                                            draw_bg +: {color: #x12161d}
                                        }
                                    }

                                    geometry_summary := View{
                                    width: Fill
                                    height: Fit
                                    flow: Down
                                    spacing: 4
                                    padding: 8
                                    draw_bg +: {color: #x171d24}
                                    show_bg: true
                                    new_batch: true

                                    geo_summary_title := Label{
                                        width: Fill
                                        height: 20
                                        text: "Geometry Summary"
                                        draw_text +: {color: #xdfe7ee}
                                    }

                                    geo_summary_grid := View{
                                        width: Fill
                                        height: Fill
                                        flow: Right
                                        spacing: 8
                                        padding: 4
                                        draw_bg +: {color: #x171d24}
                                        show_bg: true
                                        new_batch: true

                                        geo_col1 := View{
                                            width: Fill
                                            height: Fill
                                            flow: Down
                                            spacing: 4

                                            geo_length := Label{width: Fill, height: 18, text: "Length: - mm", draw_text +: {color: #xdfe7ee}}
                                            geo_bell := Label{width: Fill, height: 18, text: "Bell: - mm", draw_text +: {color: #xdfe7ee}}
                                            geo_volume := Label{width: Fill, height: 18, text: "Volume: - mm³", draw_text +: {color: #xdfe7ee}}
                                        }

                                        geo_col2 := View{
                                            width: Fill
                                            height: Fill
                                            flow: Down
                                            spacing: 4

                                            geo_taper := Label{width: Fill, height: 18, text: "Taper ratio: -", draw_text +: {color: #xdfe7ee}}
                                            geo_segments := Label{width: Fill, height: 18, text: "Segments: -", draw_text +: {color: #xdfe7ee}}
                                            geo_max_d := Label{width: Fill, height: 18, text: "Max diameter: - mm", draw_text +: {color: #xdfe7ee}}
                                        }
                                    }
                                }

                                resonance_analysis := View{
                                    width: Fill
                                    height: Fit
                                    flow: Down
                                    spacing: 4
                                    padding: 8
                                    draw_bg +: {color: #x171d24}
                                    show_bg: true
                                    new_batch: true

                                    resonance_title := Label{
                                        width: Fill
                                        height: 20
                                        text: "Resonance Analysis"
                                        draw_text +: {color: #xdfe7ee}
                                    }

                                    resonance_grid := mod.widgets.DataGrid{
                                        width: Fill
                                        height: Fit
                                        rows: 10
                                        cols: 5
                                        default_col_width: 120
                                        default_row_height: 20
                                        color_bg: #x171d24
                                        color_text: #xdfe7ee
                                    }

                                    resonance_list := Label{
                                        width: Fill
                                        height: Fit
                                        text: ""
                                        draw_text +: {font_size: 11, color: #xb0b0b0}
}
            }
        }
    }
}
                    }
                }
            preferences_modal := Modal{
                can_dismiss: true
                bg_view := View{
                    width: Fill
                    height: Fill
                    show_bg: true
                    draw_bg +: {color: #000000B3}
                }
                content := View{
                    width: Fit
                    height: Fit
                    flow: Down
                    padding: Inset{left: 20 right: 20 top: 20 bottom: 20}
                    draw_bg +: {color: #x171d24}
                    title := Label{
                        text: "Preferences"
                        draw_text +: {color: #dfe7ee, font_size: 18}
                    }
                    
                    // Theme preference
                    theme_label := Label{
                        width: Fill
                        height: 18
                        text: "Theme"
                        draw_text +: {color: #xa0a0a0}
                    }
                    theme_dropdown := DropDown{
                        width: Fill
                        height: 34
                        items: ["Dark", "Light", "Auto (System)"]
                    }
                    
                    // Volume preference
                    volume_label := Label{
                        width: Fill
                        height: 18
                        text: "Volume"
                        draw_text +: {color: #xa0a0a0}
                    }
                    volume_slider := Slider{
                        width: Fill
                        height: 40
                        min: 0.0
                        max: 1.0
                        step: 0.05
                        default: 1.0
                    }
                    volume_value := Label{
                        width: Fill
                        height: 18
                        text: "100%"
                        draw_text +: {color: #x8391a0, font_size: 11}
                    }
                    
                    // Wireframe toggle
                    wireframe_check := CheckBox{
                        width: Fit
                        height: 24
                        text: "Show Wireframe"
                    }
                    
                    // Cross-section toggle
                    cross_section_check := CheckBox{
                        width: Fit
                        height: 24
                        text: "Show Cross-Section"
                    }
                    
                    // Simulation backend preference
                    backend_label := Label{
                        width: Fill
                        height: 18
                        text: "Simulation Backend"
                        draw_text +: {color: #xa0a0a0}
                    }
                    backend_dropdown := DropDown{
                        width: Fill
                        height: 34
                        items: ["TLM (Transmission Line Model)", "Waveguide", "Complex Impedance"]
                    }
                    
                    // Buttons
                    buttons := View{
                        width: Fill
                        height: Fit
                        flow: Right
                        spacing: 10
                        align: Align{x: 1.0 y: 0.5}
                        apply_btn := Button{
                            width: 80
                            height: 34
                            text: "Apply"
                            draw_text +: {font_size: 13}
                        }
                        cancel_btn := Button{
                            width: 80
                            height: 34
                            text: "Cancel"
                            draw_text +: {font_size: 13}
                        }
                    }
                }
            }

            about_modal := Modal{
                can_dismiss: true
                bg_view := View{
                    width: Fill
                    height: Fill
                    show_bg: true
                    draw_bg +: {color: #000000B3}
                }
                content := View{
                    width: Fit
                    height: Fit
                    flow: Down
                    padding: Inset{left: 20 right: 20 top: 20 bottom: 20}
                    draw_bg +: {color: #x171d24}
                    title := Label{
                        text: "About CADSD"
                        draw_text +: {color: #dfe7ee, font_size: 18}
                    }
                    version := Label{
                        text: "Didgeridoo Analyzer v0.1"
                        draw_text +: {color: #x8391a0}
                    }
                }
            }

            documentation_modal := Modal{
                can_dismiss: true
                bg_view := View{
                    width: Fill
                    height: Fill
                    show_bg: true
                    draw_bg +: {color: #000000B3}
                }
                content := View{
                    width: Fit
                    height: Fit
                    flow: Down
                    padding: Inset{left: 20 right: 20 top: 20 bottom: 20}
                    draw_bg +: {color: #x171d24}
                    title := Label{
                        text: "Documentation"
                        draw_text +: {color: #dfe7ee, font_size: 18}
                    }
                    
                    // Documentation content scroll area
                    doc_scroll := ScrollYView{
                        width: Fill
                        height: Fit
                        max_height: 500
                        flow: Down
                        padding: 10
                        
                        // Documentation title
                        doc_title := Label{
                            width: Fill
                            height: Fit
                            text: "CADSD GUI - Complete Feature Requirements"
                            draw_text +: {color: #dfe7ee, font_size: 16, font_weight: FontWeight::Bold}
                        }
                        
                        // Documentation content (simplified preview)
                        doc_content := Label{
                            width: Fill
                            height: Fit
                            text: "UI Requirements, GUI Roadmap, Research Documentation, and Implementation Status\n\n\nClick to open full documentation in external viewer or access detailed sections."
                            draw_text +: {color: #xb0b0b0, font_size: 13}
                        }
                        
                        // Documentation buttons
                        doc_buttons := View{
                            width: Fill
                            height: Fit
                            flow: Right
                            spacing: 10
                            padding: 10
                            
                            ui_requirements_btn := Button{
                                width: 120
                                height: 34
                                text: "UI Requirements"
                                draw_text +: {font_size: 13}
                            }
                            
                            gui_roadmap_btn := Button{
                                width: 120
                                height: 34
                                text: "GUI Roadmap"
                                draw_text +: {font_size: 13}
                            }
                            
                            todo_btn := Button{
                                width: 120
                                height: 34
                                text: "TODO.md"
                                draw_text +: {font_size: 13}
                            }
                            
                            research_btn := Button{
                                width: 120
                                height: 34
                                text: "Research.md"
                                draw_text +: {font_size: 13}
                            }
                        }
                        
                        // Documentation status indicator
                        status := Label{
                            width: Fill
                            height: Fit
                            text: "✅ Documentation available - Select a section above to view details"
                            draw_text +: {color: #x2ecc71, font_size: 11}
                        }
                    }
                }
            }

            evolution_modal := Modal{
                can_dismiss: true
                bg_view := View{
                    width: Fill
                    height: Fill
                    show_bg: true
                    draw_bg +: {color: #000000B3}
                }
                content := View{
                    width: Fit
                    height: Fit
                    flow: Down
                    padding: Inset{left: 20 right: 20 top: 20 bottom: 20}
                    draw_bg +: {color: #x171d24}

                    title := Label{
                        text: "Evolution Settings"
                        draw_text +: {color: #dfe7ee, font_size: 18}
                    }

                    pop_label := Label{
                        width: Fill
                        height: 18
                        text: "Population Size"
                        draw_text +: {color: #xa0a0a0}
                    }

                    pop_slider := Slider{
                        width: Fill
                        height: 40
                        min: 10.0
                        max: 500.0
                        step: 10.0
                        default: 100.0
                    }

                    mut_label := Label{
                        width: Fill
                        height: 18
                        text: "Mutation Rate"
                        draw_text +: {color: #xa0a0a0}
                    }

                    mut_slider := Slider{
                        width: Fill
                        height: 40
                        min: 0.01
                        max: 1.0
                        step: 0.01
                        default: 0.1
                    }

                    cross_label := Label{
                        width: Fill
                        height: 18
                        text: "Crossover Rate"
                        draw_text +: {color: #xa0a0a0}
                    }

                    cross_slider := Slider{
                        width: Fill
                        height: 40
                        min: 0.1
                        max: 1.0
                        step: 0.05
                        default: 0.8
                    }

                    conv_label := Label{
                        width: Fill
                        height: 18
                        text: "Convergence Patience"
                        draw_text +: {color: #xa0a0a0}
                    }

                    conv_slider := Slider{
                        width: Fill
                        height: 40
                        min: 5.0
                        max: 100.0
                        step: 5.0
                        default: 20.0
                    }

                    strat_label := Label{
                        width: Fill
                        height: 18
                        text: "Selection Strategy"
                        draw_text +: {color: #xa0a0a0}
                    }

                    strat_dropdown := DropDown{
                        width: Fill
                        height: 28
                        labels: ["Tournament", "Roulette", "Rank"]
                        selected_item: 0
                    }

                    clone_btn := Button{
                        width: Fill
                        height: 24
                        text: "Clone from Previous"
                    }

                    button_row := View{
                        width: Fill
                        height: Fit
                        flow: Right
                        spacing: 8
                        margin: Inset{top: 16}

                        cancel_btn := Button{
                            width: Fill
                            height: 24
                            text: "Cancel"
                        }

                        apply_btn := Button{
                            width: Fill
                            height: 24
                            text: "Apply"
                            color: #x2ecc71
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
    #[live(0.0)]
    wireframe: f32,
}

impl DrawPhysMesh {
    fn apply_uniforms(&mut self, cx: &mut CxDraw) {
        let light_dir = self.light_dir.normalize();
        let fill_dir = self.fill_dir.normalize();
        self.draw_vars.set_uniform(
            cx.cx,
            live_id!(u_light_dir),
            &[light_dir.x, light_dir.y, light_dir.z],
        );
        self.draw_vars.set_uniform(
            cx.cx,
            live_id!(u_fill_dir),
            &[fill_dir.x, fill_dir.y, fill_dir.z],
        );
        self.draw_vars.set_uniform(
            cx.cx,
            live_id!(u_wireframe),
            &[self.wireframe],
        );
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
    for i in 0..rings {
        let (x, d) = segments[i];
        let radius = d * 0.05;
        let y = x * 0.1;
        if i + 1 < rings {
            let (x_next, d_next) = segments[i + 1];
            let radius_next = d_next * 0.05;
            let y_next = x_next * 0.1;
            for j in 0..24 {
                let theta = (j as f32) * std::f32::consts::TAU / 24.0;
                let theta_next = ((j + 1) as f32) * std::f32::consts::TAU / 24.0;
                let base = (i as u32) * 24 * 6 + (j as u32) * 6;

                // Triangle 1: A(1,0,0), B(0,1,0), C(0,0,1)
                // A: ring i, angle theta, barycentric UV (1,0)
                vertices.extend_from_slice(&[
                    radius * theta.cos(), y, radius * theta.sin(), theta.cos(),  // pos_nx.xyz + normal.x
                    0.0, theta.sin(), 1.0, 0.0,                                      // ny_nz_uv: normal.yz + uv (bary 1,0)
                    1.0, 1.0, 1.0, 1.0,                                                // color white
                    0.0, 0.0, 0.0, 1.0,                                                // tangent
                ]);
                // B: ring i, angle theta_next, barycentric UV (0,1)
                vertices.extend_from_slice(&[
                    radius * theta_next.cos(), y, radius * theta_next.sin(), theta_next.cos(),  // pos_nx.xyz + normal.x
                    0.0, theta_next.sin(), 0.0, 1.0,                                  // ny_nz_uv: normal.yz + uv (bary 0,1)
                    1.0, 1.0, 1.0, 1.0,                                                // color white
                    0.0, 0.0, 0.0, 1.0,                                                // tangent
                ]);
                // C: ring i+1, angle theta_next, barycentric UV (0,0)
                vertices.extend_from_slice(&[
                    radius_next * theta_next.cos(), y_next, radius_next * theta_next.sin(), theta_next.cos(),  // pos_nx.xyz + normal.x
                    0.0, theta_next.sin(), 0.0, 0.0,                                  // ny_nz_uv: normal.yz + uv (bary 0,0)
                    1.0, 1.0, 1.0, 1.0,                                                // color white
                    0.0, 0.0, 0.0, 1.0,                                                // tangent
                ]);
                indices.extend_from_slice(&[base, base + 1, base + 2]);

                // Triangle 2: A'(1,0,0), C'(0,1,0), D'(0,0,1)
                // A': ring i, angle theta, barycentric UV (1,0)
                vertices.extend_from_slice(&[
                    radius * theta.cos(), y, radius * theta.sin(), theta.cos(),  // pos_nx.xyz + normal.x
                    0.0, theta.sin(), 1.0, 0.0,                                      // ny_nz_uv: normal.yz + uv (bary 1,0)
                    1.0, 1.0, 1.0, 1.0,                                                // color white
                    0.0, 0.0, 0.0, 1.0,                                                // tangent
                ]);
                // C': ring i+1, angle theta_next, barycentric UV (0,1)
                vertices.extend_from_slice(&[
                    radius_next * theta_next.cos(), y_next, radius_next * theta_next.sin(), theta_next.cos(),  // pos_nx.xyz + normal.x
                    0.0, theta_next.sin(), 0.0, 1.0,                                 // ny_nz_uv: normal.yz + uv (bary 0,1)
                    1.0, 1.0, 1.0, 1.0,                                                // color white
                    0.0, 0.0, 0.0, 1.0,                                                // tangent
                ]);
                // D': ring i+1, angle theta, barycentric UV (0,0)
                vertices.extend_from_slice(&[
                    radius_next * theta.cos(), y_next, radius_next * theta.sin(), theta.cos(),  // pos_nx.xyz + normal.x
                    0.0, theta.sin(), 0.0, 0.0,                                      // ny_nz_uv: normal.yz + uv (bary 0,0)
                    1.0, 1.0, 1.0, 1.0,                                                // color white
                    0.0, 0.0, 0.0, 1.0,                                                // tangent
                ]);
                indices.extend_from_slice(&[base + 3, base + 4, base + 5]);
            }
        }
    }
    (indices, vertices)
}

fn make_segments(
    length: f32,
    top: f32,
    bell: f32,
    style: u32,
    bore_curve: f32,
    n: usize,
) -> Vec<(f32, f32)> {
    let geo = match style {
        0 => Geo::make_cone(length as f64, top as f64, bell as f64, n),
        1 => Geo::make_kigali(length as f64, top as f64, bell as f64, bore_curve as f64, n),
        2 => Geo::make_mbeya(length as f64, top as f64, bell as f64, bore_curve as f64, n),
        _ => Geo::make_cone(length as f64, top as f64, bell as f64, n),
    };
    geo.geo
        .iter()
        .map(|pt| (pt[0] as f32, pt[1] as f32))
        .collect()
}

fn geo_hash(geo: &Geo) -> u64 {
    let mut h: u64 = 0;
    for pt in &geo.geo {
        h = h
            .wrapping_mul(0x517cc1b727265a95)
            .wrapping_add(pt[0].to_bits());
        h = h
            .wrapping_mul(0x517cc1b727265a95)
            .wrapping_add(pt[1].to_bits());
    }
    h
}

fn create_base_geo(length: f32, top: f32, bell: f32, style: u32, bore_curve: f32, n: usize) -> Geo {
    match style {
        0 => Geo::make_cone(length as f64, top as f64, bell as f64, n),
        1 => Geo::make_kigali(length as f64, top as f64, bell as f64, bore_curve as f64, n),
        2 => Geo::make_mbeya(length as f64, top as f64, bell as f64, bore_curve as f64, n),
        _ => Geo::make_cone(length as f64, top as f64, bell as f64, n),
    }
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
        if self.initialized {
            return;
        }
        self.initialized = true;
        self.color_texture = Texture::new_with_format(
            cx,
            TextureFormat::RenderBGRAu8 {
                size: TextureSize::Auto,
                initial: true,
            },
        );
        self.depth_texture = Texture::new_with_format(
            cx,
            TextureFormat::DepthD32 {
                size: TextureSize::Auto,
                initial: true,
            },
        );
        self.pass.set_color_texture(
            cx,
            &self.color_texture,
            DrawPassClearColor::ClearWith(self.clear_color),
        );
        self.pass
            .set_depth_texture(cx, &self.depth_texture, DrawPassClearDepth::ClearWith(1.0));
        cx.passes[self.pass.draw_pass_id()].keep_camera_matrix = true;

        self.camera.orbit_yaw = 0.72;
        self.camera.orbit_pitch = -0.34;

        let segs = make_segments(950.0, 35.0, 85.0, 0, 0.0, 50);
        let (indices, vertices) = build_bore_geometry(&segs);
        let geometry = Geometry::new(cx);
        geometry.update(cx, indices, vertices);
        self.geometry = Some(geometry);
        self.last_hash = 0;
    }

    pub fn update_bore(
        &mut self,
        cx: &mut Cx,
        length: f32,
        top: f32,
        bell: f32,
        style: u32,
        bore_curve: f32,
        n: usize,
    ) {
        let geo = create_base_geo(length, top, bell, style, bore_curve, n);
        self.update_bore_geo(cx, &geo);
    }

    pub fn update_bore_geo(&mut self, cx: &mut Cx, geo: &Geo) {
        let hash = geo_hash(geo);
        if hash == self.last_hash {
            return;
        }
        self.last_hash = hash;

        let segs: Vec<(f32, f32)> = geo
            .geo
            .iter()
            .map(|pt| (pt[0] as f32, pt[1] as f32))
            .collect();
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
    fn widget_uid(&self) -> WidgetUid {
        self.uid
    }
    fn walk(&mut self, _cx: &mut Cx) -> Walk {
        self.walk
    }
    fn area(&self) -> Area {
        self.area
    }
    fn redraw(&mut self, cx: &mut Cx) {
        self.area.redraw(cx);
    }
}

impl Widget for BoreViewport {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        self.camera.handle_desktop_interaction(cx, event);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.walk_turtle_with_area(&mut self.area, walk);
        if rect.size.x <= 1.0 || rect.size.y <= 1.0 {
            return DrawStep::done();
        }

        self.ensure_initialized(cx.cx);
        self.camera.set_desktop_viewport_rect(rect);
        self.pass.set_size(cx, rect.size);
        self.pass.set_color_texture(
            cx,
            &self.color_texture,
            DrawPassClearColor::ClearWith(self.clear_color),
        );
        self.pass
            .set_depth_texture(cx, &self.depth_texture, DrawPassClearDepth::ClearWith(1.0));

        cx.make_child_pass(&self.pass);
        cx.begin_pass(&self.pass, None);
        if let Some(scene_state) = self.camera.desktop_scene_state(rect, cx.time()) {
            set_pass_camera(cx.cx, &self.pass, &scene_state);
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
    #[rust]
    impedance_data: Vec<DataPoint>,
    #[rust]
    fundamental_freq: f32,
    #[rust]
    sim_rx: Option<mpsc::Receiver<SimResult>>,
    #[rust]
    peaks: Vec<(f64, f64)>,
    #[rust]
    geo_length: f32,
    #[rust]
    geo_bell: f32,
    #[rust]
    geo_volume: f32,
    #[rust]
    geo_taper: f32,
    #[rust]
    geo_segments: f32,
    #[rust]
    geo_max_d: f32,
    #[rust]
    segments_list: Vec<[f32; 2]>,
    #[rust]
    show_segment_editor: bool,
    #[rust]
    current_view: String,
    #[rust]
    show_wireframe: bool,
    #[rust]
    show_cross_section: bool,
    #[rust]
    show_sidebar: bool,
    #[rust]
    is_fullscreen: bool,
    #[rust]
    theme_mode: String,
    #[rust]
    current_geo: Option<Geo>,
    #[rust]
    bubbles: Vec<(f32, f32, f32)>,
    #[rust]
    bubble_count: usize,
    #[rust]
    enable_mouthpiece: bool,
    #[rust]
    mouthpiece_type: String,
    #[rust]
    mouthpiece_length: f32,
    #[rust]
    mouthpiece_diameter: f32,
    #[rust]
    enable_holes: bool,
    #[rust]
    hole_count: usize,
    #[rust]
    hole_positions: Vec<f32>,
    #[rust]
    hole_diameters: Vec<f32>,
    #[rust]
    weight_fundamental: f64,
    #[rust]
    weight_harmonics: f64,
    #[rust]
    weight_peaks: f64,
    #[rust]
    loss_fundamental_value: f64,
    #[rust]
    loss_harmonics_value: f64,
    #[rust]
    loss_peaks_value: f64,
    #[rust]
    tairua_loss_value: f64,
    #[rust]
    target_freq: f64,
    #[rust]
    top_diameter: f32,
    #[rust]
    bore_style_name: String,
    #[rust]
    cross_section_data: Vec<(f32, f32)>,
    #[rust]
    loss_total_text: String,
    #[rust]
    loss_fundamental_text: String,
    #[rust]
    loss_harmonics_text: String,
    #[rust]
    loss_peaks_text: String,
    #[rust]
    optimization_running: bool,
    #[rust]
    optimization_target_freq: f64,
    #[rust]
    optimization_progress: f64,
    #[rust]
    optimization_status_text: String,
    #[rust]
    optimization_best_top: f32,
    #[rust]
    optimization_best_bell: f32,
    #[rust]
    optimization_best_style: u32,
    #[rust]
    optimization_best_history: Vec<GenerationLoss>,
    #[rust]
    optimization_stop_flag: Option<Arc<AtomicBool>>,
    #[rust]
    opt_rx: Option<mpsc::Receiver<OptProgress>>,
    #[rust]
    history: Vec<ProjectState>,
    #[rust]
    history_index: usize,
    #[rust(100)]
    evo_population_size: usize,
    #[rust(0.1)]
    evo_mutation_rate: f64,
    #[rust(0.8)]
    evo_crossover_rate: f64,
    #[rust(20)]
    evo_convergence_patience: usize,
    #[rust(0)]
    evo_selection_strategy: usize,
    #[rust(false)]
    evo_clone_from_previous: bool,
    #[rust(100)]
    evo_population_size_original: usize,
    #[rust(0.1)]
    evo_mutation_rate_original: f64,
    #[rust(0.8)]
    evo_crossover_rate_original: f64,
    #[rust(20)]
    evo_convergence_patience_original: usize,
    #[rust(0)]
    evo_selection_strategy_original: usize,
    #[rust(false)]
    evo_clone_from_previous_original: bool,
    #[rust(false)]
    show_preferences: bool,
    #[rust(false)]
    show_documentation: bool,
    #[rust("dark".to_string())]
    pref_theme_mode: String,
    #[rust(100.0f32)]
    pref_volume: f32,
    #[rust(true)]
    pref_show_wireframe: bool,
    #[rust(true)]
    pref_show_cross_section: bool,
    #[rust("Tlm".to_string())]
    pref_simulation_backend: String,
    #[rust("".to_string())]
    documentation_text: String,
    #[rust(None)]
    project_path: Option<PathBuf>,
}

#[allow(dead_code)]
struct SimResult {
    data: Vec<DataPoint>,
    fundamental: f64,
    note: String,
    resonance_count: usize,
    peaks: Vec<(f64, f64)>,
}

#[allow(dead_code)]
struct OptProgress {
    progress: f64,
    status: String,
    best_top: f32,
    best_bell: f32,
    best_style: u32,
    total_loss: f64,
}

#[derive(Clone)]
struct GenerationLoss {
    generation: usize,
    total_loss: f64,
    fundamental_loss: f64,
    harmonic_loss: f64,
    peak_loss: f64,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ProjectState {
    geo: Geo,
    bubbles: Vec<(f32, f32, f32)>,
    length: f32,
    top: f32,
    bell: f32,
    style: u32,
    bore_curve: f32,
    segments: usize,
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let mut needs_viewport_update = false;
        let mut length = 950.0;
        let mut top = 35.0;
        let mut bell = 85.0;
        let mut style = 0u32;
        let mut bore_curve = 1.0f32;
        let mut segments = 50usize;

        if let Some(v) = self.ui.slider(cx, ids!(length_slider)).slided(actions) {
            self.push_current_state();
            length = v;
            self.ui
                .label(cx, ids!(length_value))
                .set_text(cx, &format!("{:.0}", v));
            needs_viewport_update = true;
        }
        if let Some(v) = self.ui.slider(cx, ids!(top_slider)).slided(actions) {
            self.push_current_state();
            top = v;
            self.ui
                .label(cx, ids!(top_value))
                .set_text(cx, &format!("{:.1}", v));
            needs_viewport_update = true;
        }
        if let Some(v) = self.ui.slider(cx, ids!(bell_slider)).slided(actions) {
            self.push_current_state();
            bell = v;
            self.ui
                .label(cx, ids!(bell_value))
                .set_text(cx, &format!("{:.1}", v));
            needs_viewport_update = true;
        }
        if let Some(v) = self.ui.slider(cx, ids!(segments_slider)).slided(actions) {
            self.push_current_state();
            segments = v as usize;
            self.ui
                .label(cx, ids!(segments_value))
                .set_text(cx, &format!("{}", segments));
            needs_viewport_update = true;
        }
        if let Some(v) = self.ui.slider(cx, ids!(bore_curve_slider)).slided(actions) {
            self.push_current_state();
            bore_curve = v as f32;
            self.ui
                .label(cx, ids!(bore_curve_value))
                .set_text(cx, &format!("{:.1}", v));
            needs_viewport_update = true;
        }
        if let Some(v) = self
            .ui
            .drop_down(cx, ids!(bore_style_dropdown))
            .selected(actions)
        {
            self.push_current_state();
            style = v as u32;
            needs_viewport_update = true;
        }
        let profile_name = match style {
            0 => "Cone",
            1 => "Kigali",
            2 => "Mbeya",
            _ => "Cone",
        };
        self.ui
            .label(cx, ids!(geo_profile))
            .set_text(cx, &format!("Profile: {}", profile_name));

        if self.ui.button(cx, ids!(bubble_add_button)).clicked(actions) {
            self.push_current_state();
            let pos_str = self.ui.text_input(cx, ids!(bubble_pos_input)).text();
            let width_str = self.ui.text_input(cx, ids!(bubble_width_input)).text();
            let height_str = self.ui.text_input(cx, ids!(bubble_height_input)).text();
            if let (Ok(pos), Ok(width), Ok(height)) = (
                pos_str.parse::<f32>(),
                width_str.parse::<f32>(),
                height_str.parse::<f32>(),
            ) {
                self.bubbles.push((pos, width, height));
                self.bubble_count = self.bubbles.len();
                needs_viewport_update = true;
            }
        }
        if self
            .ui
            .button(cx, ids!(bubble_remove_button))
            .clicked(actions)
        {
            self.push_current_state();
            self.bubbles.pop();
            self.bubble_count = self.bubbles.len();
            needs_viewport_update = true;
        }
        if self.ui.button(cx, ids!(seg_move_button)).clicked(actions) {
            self.push_current_state();
            if let Some(ref mut geo) = self.current_geo {
                let start_str = self.ui.text_input(cx, ids!(seg_start_input)).text();
                let end_str = self.ui.text_input(cx, ids!(seg_end_input)).text();
                let offset_str = self.ui.text_input(cx, ids!(seg_offset_input)).text();
                if let (Ok(start), Ok(end), Ok(offset)) = (
                    start_str.parse::<usize>(),
                    end_str.parse::<usize>(),
                    offset_str.parse::<f64>(),
                ) {
                    geo.move_segments_x(start, end, offset);
                    needs_viewport_update = true;
                }
            }
        }
        if self.ui.button(cx, ids!(seg_sort_button)).clicked(actions) {
            self.push_current_state();
            if let Some(ref mut geo) = self.current_geo {
                geo.sort_segments();
                needs_viewport_update = true;
            }
        }
        if self
            .ui
            .button(cx, ids!(segment_editor_toggle))
            .clicked(actions)
        {
            self.push_current_state();
            self.show_segment_editor = !self.show_segment_editor;
            let btn = self.ui.button(cx, ids!(segment_editor_toggle));
            btn.set_text(
                cx,
                if self.show_segment_editor {
                    "Hide Segment Editor"
                } else {
                    "Show Segment Editor"
                },
            );
        }

        // Handle mouthpiece controls
        if self.ui.button(cx, ids!(mp_toggle)).clicked(actions) {
            self.push_current_state();
            self.enable_mouthpiece = !self.enable_mouthpiece;
            let btn = self.ui.button(cx, ids!(mp_toggle));
            btn.set_text(
                cx,
                if self.enable_mouthpiece {
                    "Disable Mouthpiece"
                } else {
                    "Enable Mouthpiece"
                },
            );
            needs_viewport_update = true;
        }
        if self.enable_mouthpiece {
            if let Some(v) = self
                .ui
                .drop_down(cx, ids!(mp_type_dropdown))
                .selected(actions)
            {
                self.push_current_state();
                let types = ["None", "Reed", "Embouchure", "Fipple", "Cup"];
                self.mouthpiece_type = types[v].to_string();
                needs_viewport_update = true;
            }
            if let Some(v) = self.ui.slider(cx, ids!(mp_length_slider)).slided(actions) {
                self.mouthpiece_length = v as f32;
                self.ui
                    .label(cx, ids!(mp_length_value))
                    .set_text(cx, &format!("Length: {:.0} mm", v));
                needs_viewport_update = true;
            }
            if let Some(v) = self.ui.slider(cx, ids!(mp_diameter_slider)).slided(actions) {
                self.mouthpiece_diameter = v as f32;
                self.ui
                    .label(cx, ids!(mp_diameter_value))
                    .set_text(cx, &format!("Diameter: {:.0} mm", v));
                needs_viewport_update = true;
            }
        }

        // Handle hole controls
        if self.ui.button(cx, ids!(holes_toggle)).clicked(actions) {
            self.push_current_state();
            self.enable_holes = !self.enable_holes;
            let btn = self.ui.button(cx, ids!(holes_toggle));
            btn.set_text(
                cx,
                if self.enable_holes {
                    "Disable Holes"
                } else {
                    "Enable Holes"
                },
            );
            needs_viewport_update = true;
        }
        if self.enable_holes {
if let Some(v) = self.ui.slider(cx, ids!(holes_count_slider)).slided(actions) {
            self.push_current_state();
            self.hole_count = v as usize;
                self.ui
                    .label(cx, ids!(holes_count_label))
                    .set_text(cx, &format!("Hole count: {}", v));
                needs_viewport_update = true;
            }
        }
        // Update holes list display
        if self.hole_count > 0 && self.hole_positions.len() < self.hole_count {
            while self.hole_positions.len() < self.hole_count {
                self.hole_positions.push(0.0);
                self.hole_diameters.push(5.0);
            }
        } else if self.hole_count == 0 {
            self.hole_positions = vec![];
            self.hole_diameters = vec![];
        }
        let hole_text: String = if self.hole_count > 0 {
            format!("Holes: {} active", self.hole_count)
        } else {
            "Holes: none".to_string()
        };
        self.ui.label(cx, ids!(holes_list)).set_text(cx, &hole_text);

        // Handle loss breakdown controls
        if let Some(v) = self.ui.slider(cx, ids!(loss_fund_slider)).slided(actions) {
            self.weight_fundamental = v;
            needs_viewport_update = true;
        }
        if let Some(v) = self.ui.slider(cx, ids!(loss_harm_slider)).slided(actions) {
            self.weight_harmonics = v;
            needs_viewport_update = true;
        }
        if let Some(v) = self.ui.slider(cx, ids!(loss_peak_slider)).slided(actions) {
            self.weight_peaks = v;
            needs_viewport_update = true;
        }
        if let Some(v) = self
            .ui
            .text_input(cx, ids!(loss_target_input))
            .changed(actions)
        {
            if let Ok(freq) = v.parse::<f64>() {
                self.target_freq = freq;
                needs_viewport_update = true;
            }
        }

        // Handle export buttons
        if self.ui.button(cx, ids!(export_csv_button)).clicked(actions) {
            self.export_impedance_csv();
        }
        if self
            .ui
            .button(cx, ids!(export_json_button))
            .clicked(actions)
        {
            self.export_geometry_json();
        }

        // Handle optimization controls
        if self.ui.button(cx, ids!(opt_start_button)).clicked(actions) {
            if self.opt_rx.is_some() {
                return;
            }
            self.optimization_running = true;
            self.optimization_progress = 0.0;
            self.optimization_status_text = "Starting...".to_string();
            self.optimization_target_freq = self.target_freq;
            self.optimization_best_history.clear();
            self.ui
                .label(cx, ids!(opt_progress_label))
                .set_text(cx, "Status: starting");
            self.ui
                .label(cx, ids!(opt_target_label))
                .set_text(cx, &format!("Target Freq: {:.1} Hz", self.target_freq));

            let (tx, rx) = mpsc::channel();
            self.opt_rx = Some(rx);
            let stop_flag = Arc::new(AtomicBool::new(false));
            self.optimization_stop_flag = Some(stop_flag.clone());

let opt_geo = if let Some(ref geo) = self.current_geo {
    geo.copy()
} else {
    create_base_geo(950.0, 35.0, 85.0, 0, 0.0, 50)
};
let target_freq = self.optimization_target_freq;
let weight_fundamental = self.weight_fundamental;
let weight_harmonics = self.weight_harmonics;
let weight_peaks = self.weight_peaks;
let evo_population_size = self.evo_population_size;
let evo_mutation_rate = self.evo_mutation_rate;
let evo_crossover_rate = self.evo_crossover_rate;

            std::thread::spawn(move || {
                let loss_fn = TairuaLoss::new()
                    .with_target_frequency(target_freq)
                    .with_weights(weight_fundamental, weight_harmonics, weight_peaks);

                let pop_size = evo_population_size;
                let mutation_rate = evo_mutation_rate;
                let generations = 30usize; // TODO: make configurable

                let mut population = Vec::new();
                for _ in 0..pop_size {
                    let genes = vec![
                        opt_geo.length() + (rand::random::<f64>() - 0.5) * 400.0,
                        opt_geo.geo.first().map(|s| s[1]).unwrap_or(35.0)
                            + (rand::random::<f64>() - 0.5) * 20.0,
                        opt_geo.bellsize() + (rand::random::<f64>() - 0.5) * 40.0,
                        (rand::random::<f64>() * 40.0).max(5.0).min(50.0),
                    ];
                    population.push(GeoGenome::new(genes));
                }

                let evolver = Nuevolution::new(pop_size, generations)
                    .set_mutation_rate(mutation_rate)
                    .set_crossover_rate(evo_crossover_rate)
                    .set_elite_size(2)
                    .set_verbose(false);

                let stop = stop_flag.clone();
                let progress_tx = tx.clone();

                let result = evolver.evolve(
                    population,
                    &LossFunctionType::TairuaLoss(loss_fn),
Some(&|gen: usize, best_fitness: f64| {
                        if stop.load(Ordering::Relaxed) {
                            return;
                        }
                        let _ = progress_tx.send(OptProgress {
                            progress: gen as f64 / generations as f64,
                            status: format!("Gen {}/{} best loss {:.4}", gen, generations, best_fitness),
                            best_top: 0.0,
                            best_bell: 0.0,
                            best_style: 0,
                            total_loss: best_fitness,
                        });
                    }),
                );

                match result {
                    Ok(final_pop) => {
                        if let Some(best) = final_pop.first() {
                            let best_geo = best.to_geo();
                            let best_top = best_geo.geo.first().map(|s| s[1]).unwrap_or(35.0) as f32;
                            let best_bell = best_geo.bellsize() as f32;
                            let best_fitness = best.fitness.unwrap_or(f64::INFINITY);
                            let _ = progress_tx.send(OptProgress {
                                progress: 1.0,
                                status: format!("Done. Best loss: {:.4}", best_fitness),
                                best_top,
                                best_bell,
                                best_style: 0,
                                total_loss: best_fitness,
                            });
                        }
                    }
                    Err(e) => {
                        let _ = progress_tx.send(OptProgress {
                            progress: 0.0,
                            status: format!("Error: {}", e),
                            best_top: 0.0,
                            best_bell: 0.0,
                            best_style: 0,
                            total_loss: f64::INFINITY,
                        });
                    }
                }
            });
        }

        if self.ui.button(cx, ids!(opt_stop_button)).clicked(actions) {
            if let Some(flag) = &self.optimization_stop_flag {
                flag.store(true, Ordering::Relaxed);
            }
            self.optimization_running = false;
            self.optimization_status_text = "Stopped".to_string();
            self.ui
                .label(cx, ids!(opt_progress_label))
                .set_text(cx, "Status: stopped");
        }

        if self.ui.button(cx, ids!(opt_settings_button)).clicked(actions) {
            // Save original values for Cancel functionality
            self.evo_population_size_original = self.evo_population_size;
            self.evo_mutation_rate_original = self.evo_mutation_rate;
            self.evo_crossover_rate_original = self.evo_crossover_rate;
            self.evo_convergence_patience_original = self.evo_convergence_patience;
            self.evo_selection_strategy_original = self.evo_selection_strategy;
            self.evo_clone_from_previous_original = self.evo_clone_from_previous;

            // Show current values when the modal opens
            self.ui
                .label(cx, ids!(pop_label))
                .set_text(cx, &format!("Population Size: {}", self.evo_population_size));
            self.ui
                .label(cx, ids!(mut_label))
                .set_text(cx, &format!("Mutation Rate: {:.2}", self.evo_mutation_rate));
            self.ui
                .label(cx, ids!(cross_label))
                .set_text(cx, &format!("Crossover Rate: {:.2}", self.evo_crossover_rate));
            self.ui
                .label(cx, ids!(conv_label))
                .set_text(cx, &format!("Convergence Patience: {}", self.evo_convergence_patience));
            let strategy_names = ["Tournament", "Roulette", "Rank"];
            self.ui
                .label(cx, ids!(strat_label))
                .set_text(
                    cx,
                    &format!(
                        "Selection Strategy: {}",
                        strategy_names[self.evo_selection_strategy]
                    ),
                );
            let clone_label = if self.evo_clone_from_previous {
                "Clone from Previous (On)"
            } else {
                "Clone from Previous"
            };
            self.ui.button(cx, ids!(clone_btn)).set_text(cx, clone_label);
            self.ui.modal(cx, ids!(evolution_modal)).open(cx);
        }

        // Evolution modal slider handlers
        if let Some(pop) = self.ui.slider(cx, ids!(pop_slider)).slided(actions) {
            self.evo_population_size = pop as usize;
            self.ui
                .label(cx, ids!(pop_label))
                .set_text(cx, &format!("Population Size: {}", self.evo_population_size));
        }
        if let Some(mutation_rate) = self.ui.slider(cx, ids!(mut_slider)).slided(actions) {
            self.evo_mutation_rate = mutation_rate;
            self.ui
                .label(cx, ids!(mut_label))
                .set_text(cx, &format!("Mutation Rate: {:.2}", self.evo_mutation_rate));
        }
        if let Some(cross) = self.ui.slider(cx, ids!(cross_slider)).slided(actions) {
            self.evo_crossover_rate = cross;
            self.ui
                .label(cx, ids!(cross_label))
                .set_text(cx, &format!("Crossover Rate: {:.2}", self.evo_crossover_rate));
        }
        if let Some(conv) = self.ui.slider(cx, ids!(conv_slider)).slided(actions) {
            self.evo_convergence_patience = conv as usize;
            self.ui
                .label(cx, ids!(conv_label))
                .set_text(cx, &format!("Convergence Patience: {}", self.evo_convergence_patience));
        }
        if let Some(strat) = self.ui.drop_down(cx, ids!(strat_dropdown)).selected(actions) {
            self.evo_selection_strategy = strat;
            let strategy_names = ["Tournament", "Roulette", "Rank"];
            self.ui
                .label(cx, ids!(strat_label))
                .set_text(
                    cx,
                    &format!(
                        "Selection Strategy: {}",
                        strategy_names[self.evo_selection_strategy]
                    ),
                );
        }

        // Clone button handler
        if self.ui.button(cx, ids!(clone_btn)).clicked(actions) {
            self.evo_clone_from_previous = !self.evo_clone_from_previous;
            let clone_label = if self.evo_clone_from_previous {
                "Clone from Previous (On)"
            } else {
                "Clone from Previous"
            };
            self.ui.button(cx, ids!(clone_btn)).set_text(cx, clone_label);
        }

        if self.ui.button(cx, ids!(apply_btn)).clicked(actions) {
            // Apply: keep current slider/dropdown values (already applied via handlers)
            let clone_label = if self.evo_clone_from_previous {
                "Clone from Previous (On)"
            } else {
                "Clone from Previous"
            };
            self.ui.button(cx, ids!(clone_btn)).set_text(cx, clone_label);
            self.ui.modal(cx, ids!(evolution_modal)).close(cx);
        }

        if self.ui.button(cx, ids!(cancel_btn)).clicked(actions) {
            // Cancel: restore original values
            self.evo_population_size = self.evo_population_size_original;
            self.evo_mutation_rate = self.evo_mutation_rate_original;
            self.evo_crossover_rate = self.evo_crossover_rate_original;
            self.evo_convergence_patience = self.evo_convergence_patience_original;
            self.evo_selection_strategy = self.evo_selection_strategy_original;
            self.evo_clone_from_previous = self.evo_clone_from_previous_original;

            // Restore labels to original values
            self.ui
                .label(cx, ids!(pop_label))
                .set_text(cx, &format!("Population Size: {}", self.evo_population_size));
            self.ui
                .label(cx, ids!(mut_label))
                .set_text(cx, &format!("Mutation Rate: {:.2}", self.evo_mutation_rate));
            self.ui
                .label(cx, ids!(cross_label))
                .set_text(cx, &format!("Crossover Rate: {:.2}", self.evo_crossover_rate));
            self.ui
                .label(cx, ids!(conv_label))
                .set_text(cx, &format!("Convergence Patience: {}", self.evo_convergence_patience));
            let strategy_names = ["Tournament", "Roulette", "Rank"];
            self.ui
                .label(cx, ids!(strat_label))
                .set_text(
                    cx,
                    &format!(
                        "Selection Strategy: {}",
                        strategy_names[self.evo_selection_strategy]
                    ),
                );
            let clone_label = if self.evo_clone_from_previous {
                "Clone from Previous (On)"
            } else {
                "Clone from Previous"
            };
            self.ui.button(cx, ids!(clone_btn)).set_text(cx, clone_label);
            self.ui.modal(cx, ids!(evolution_modal)).close(cx);
        }

        // Drain optimization progress channel
        if let Some(rx) = &mut self.opt_rx {
            while let Ok(progress) = rx.try_recv() {
                self.optimization_progress = progress.progress;
                self.optimization_status_text = progress.status;
                self.optimization_best_top = progress.best_top;
                self.optimization_best_bell = progress.best_bell;
                self.optimization_best_style = progress.best_style;
                // Accumulate generational loss history
                self.optimization_best_history.push(GenerationLoss {
                    generation: self.optimization_best_history.len(),
                    total_loss: progress.total_loss,
                    fundamental_loss: 0.0,
                    harmonic_loss: 0.0,
                    peak_loss: 0.0,
                });
            }
            if self.optimization_progress >= 1.0 || !self.optimization_running && self.optimization_progress > 0.0 {
                self.opt_rx = None;
                self.optimization_stop_flag = None;
                self.optimization_running = false;
            }
        }

        // Update optimization UI labels
        self.ui
            .label(cx, ids!(opt_progress_label))
            .set_text(cx, &format!("Status: {}", self.optimization_status_text));
        self.ui.label(cx, ids!(opt_target_label)).set_text(
            cx,
            &format!(
                "Target Freq: {:.1} Hz | Progress: {:.0}%",
                self.optimization_target_freq,
                self.optimization_progress * 100.0
            ),
        );

        // Update generational loss chart data
        if let Some(mut chart) = self
            .ui
            .widget(cx, ids!(generational_loss_chart))
            .borrow_mut::<LineChart>()
        {
            let data_points: Vec<makepad_widgets::chart::DataPoint> = self
                .optimization_best_history
                .iter()
                .enumerate()
                .map(|(i, gl)| makepad_widgets::chart::DataPoint {
                    x: i as f64,
                    y: gl.total_loss,
                })
                .collect();
            chart.set_data(data_points);
        }

        // Update segments list and label from current_geo
        if let Some(ref geo) = self.current_geo {
            let segs: Vec<[f32; 2]> = geo
                .geo
                .iter()
                .map(|pt| [pt[0] as f32, pt[1] as f32])
                .collect();
            self.segments_list = segs;
        }
        let seg_text: String = self
            .segments_list
            .iter()
            .enumerate()
            .map(|(_i, s)| format!("[{:.0}, {:.0}]", s[0], s[1]))
            .collect::<Vec<_>>()
            .join("  ");
        self.ui
            .label(cx, ids!(segment_editor_list))
            .set_text(cx, &format!("{}: {}", self.segments_list.len(), seg_text));

        self.ui
            .label(cx, ids!(bubble_count))
            .set_text(cx, &format!("Bubbles: {}", self.bubble_count));

        if needs_viewport_update {
            if let Some(mut vp) = self
                .ui
                .widget(cx, ids!(viewport))
                .borrow_mut::<BoreViewport>()
            {
                let mut geo = create_base_geo(
                    length as f32,
                    top as f32,
                    bell as f32,
                    style,
                    bore_curve,
                    segments,
                );
                for &(pos, width, height) in &self.bubbles {
                    geo.make_bubble(pos as f64, width as f64, height as f64);
                }
                self.current_geo = Some(geo.copy());
                vp.update_bore_geo(cx, self.current_geo.as_ref().unwrap());
            }
            self.geo_length = length as f32;
            self.geo_bell = bell as f32;
            self.top_diameter = top as f32;
            self.bore_style_name = match style {
                0 => "Cone".to_string(),
                1 => "Kigali".to_string(),
                2 => "Mbeya".to_string(),
                _ => "Cone".to_string(),
            };
            let max_d = bell.max(top);
            self.geo_max_d = max_d as f32;
            let taper = if top > 0.0 && bell > 0.0 {
                max_d / top.min(bell)
            } else {
                1.0
            };
            self.geo_taper = taper as f32;
            self.geo_segments = segments as f32;
            // Volume formula matches Geo::compute_volume: PI * (d1² + d1*d2 + d2²) * L / 12
            // where d1=top, d2=bell are diameters (not radii)
            let d1 = top as f64;
            let d2 = bell as f64;
            let h = length as f64;
            self.geo_volume =
                (std::f64::consts::PI * (d1 * d1 + d1 * d2 + d2 * d2) * h / 12.0) as f32;
        }

        if let Some(rx) = self.sim_rx.take() {
            match rx.try_recv() {
                Ok(result) => {
                    self.impedance_data = result.data;
                    self.fundamental_freq = result.fundamental as f32;
                    self.peaks = result.peaks;
                    if let Some(mut chart) = self
                        .ui
                        .widget(cx, ids!(impedance_chart))
                        .borrow_mut::<LineChart>()
                    {
                        chart.set_data(self.impedance_data.clone());
                    }
                    self.ui.label(cx, ids!(fundamental_label)).set_text(
                        cx,
                        &format!(
                            "Fundamental: {:.1} Hz ({})",
                            result.fundamental, result.note
                        ),
                    );
                    self.ui
                        .label(cx, ids!(resonances_label))
                        .set_text(cx, &format!("Resonances: {}", result.resonance_count));
                    self.ui.label(cx, ids!(running_label)).set_text(cx, "Ready");

                    self.ui
                        .label(cx, ids!(geo_length))
                        .set_text(cx, &format!("Length: {:.0} mm", length));
                    self.ui
                        .label(cx, ids!(geo_bell))
                        .set_text(cx, &format!("Bell: {:.1} mm", bell));
                    self.ui
                        .label(cx, ids!(geo_volume))
                        .set_text(cx, &format!("Volume: {:.0} mm³", self.geo_volume));
                    self.ui
                        .label(cx, ids!(geo_taper))
                        .set_text(cx, &format!("Taper ratio: {:.2}", self.geo_taper));
                    self.ui
                        .label(cx, ids!(geo_segments))
                        .set_text(cx, &format!("Segments: {:.0}", self.geo_segments));
                    self.ui
                        .label(cx, ids!(geo_max_d))
                        .set_text(cx, &format!("Max diameter: {:.1} mm", self.geo_max_d));

                    let resonance_text: String = self
                        .peaks
                        .iter()
                        .take(10)
                        .map(|(f, z)| {
                            let note = note_name(freq_to_note(*f));
                            format!("{:.1} Hz ({:.0} Pa, {note})\n", f, z)
                        })
                        .collect();
                    self.ui
                        .label(cx, ids!(resonance_list))
                        .set_text(cx, &resonance_text);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.sim_rx = Some(rx);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.ui
                        .label(cx, ids!(running_label))
                        .set_text(cx, "Simulation failed");
                }
            }
        }

        self.update_loss_values();
        self.ui
            .label(cx, ids!(loss_total))
            .set_text(cx, &self.loss_total_text);
        self.ui
            .label(cx, ids!(loss_w_fund))
            .set_text(cx, &self.loss_fundamental_text);
        self.ui
            .label(cx, ids!(loss_w_harm))
            .set_text(cx, &self.loss_harmonics_text);
        self.ui
            .label(cx, ids!(loss_w_peak))
            .set_text(cx, &self.loss_peaks_text);

        // Update loss breakdown BarChart with per-component loss values
        let loss_chart_data: Vec<DataPoint> = vec![
            DataPoint {
                x: 0.0,
                y: self.loss_fundamental_value,
            },
            DataPoint {
                x: 1.0,
                y: self.loss_harmonics_value,
            },
            DataPoint {
                x: 2.0,
                y: self.loss_peaks_value,
            },
            DataPoint {
                x: 3.0,
                y: self.tairua_loss_value,
            },
        ];
        if let Some(mut chart) = self
            .ui
            .widget(cx, ids!(loss_chart))
            .borrow_mut::<BarChart>()
        {
            chart.set_data(loss_chart_data);
        }

        self.update_cross_section_data();

        // Update cross-section plot
        if let Some(mut chart) = self
            .ui
            .widget(cx, ids!(cross_section_plot))
            .borrow_mut::<LineChart>()
        {
            let data_points: Vec<DataPoint> = self
                .cross_section_data
                .iter()
                .map(|(x, y)| DataPoint {
                    x: *x as f64,
                    y: *y as f64,
                })
                .collect();
            chart.set_data(data_points);
        }

        if self.ui.button(cx, ids!(run_button)).clicked(actions) {
            if self.sim_rx.is_some() {
                return;
            }
            self.ui
                .label(cx, ids!(running_label))
                .set_text(cx, "Simulating...");
            let (tx, rx) = mpsc::channel();
            self.sim_rx = Some(rx);

            // Clone the current geometry for the simulation thread; fallback to default if none set
            let sim_geo = if let Some(ref geo) = self.current_geo {
                geo.copy()
            } else {
                create_base_geo(950.0, 35.0, 85.0, 0, 0.0, 50)
            };

            std::thread::spawn(move || {
                let geo = sim_geo; // Use the pre-computed geometry with bubbles
                let freqs = get_log_simulation_frequencies();
                let impedances = match acoustical_simulation(&geo, &freqs, "tlm_python") {
                    Ok(impedances) => impedances,
                    Err(_) => {
                        let _ = tx.send(SimResult {
                            data: Vec::new(),
                            fundamental: 0.0,
                            note: "Simulation error".to_string(),
                            resonance_count: 0,
                            peaks: Vec::new(),
                        });
                        return;
                    }
                };

                let peaks: Vec<(f64, f64)> = freqs
                    .iter()
                    .zip(impedances.iter())
                    .enumerate()
                    .filter_map(|(i, (f, z))| {
                        if i > 0
                            && i + 1 < impedances.len()
                            && *z > impedances[i - 1]
                            && *z > impedances[i + 1]
                        {
                            Some((*f, *z))
                        } else {
                            None
                        }
                    })
                    .collect();

                let fundamental = peaks
                    .iter()
                    .find(|(f, _)| *f > 20.0)
                    .map(|(f, _)| *f)
                    .unwrap_or(0.0);
                let note = if fundamental > 0.0 {
                    note_name(freq_to_note(fundamental))
                } else {
                    "—".to_string()
                };

                let data: Vec<DataPoint> = freqs
                    .iter()
                    .zip(impedances.iter())
                    .map(|(f, z)| DataPoint { x: *f, y: *z })
                    .collect();

                let _ = tx.send(SimResult {
                    data,
                    fundamental,
                    note,
                    resonance_count: peaks.len(),
                    peaks,
                });
            });
        }

        // Handle menu bar DropDown selections
        if let Some(idx) = self.ui.drop_down(cx, ids!(file_menu)).selected(actions) {
            match idx {
                0 => {
                    // New Project — reset to default geometry
                    self.current_geo = None;
                    self.bubbles = vec![];
                    self.bubble_count = 0;
                    self.show_segment_editor = false;
                    if let Some(mut vp) = self.ui.widget(cx, ids!(viewport)).borrow_mut::<BoreViewport>() {
                        vp.update_bore_geo(cx, &Geo::make_cone(950.0, 35.0, 85.0, 50));
                    }
                    self.geo_length = 950.0;
                    self.geo_bell = 85.0;
                    self.top_diameter = 35.0;
                    self.bore_style_name = "Cone".to_string();
                    self.geo_segments = 50.0;
                    self.geo_taper = 1.0;
                    let d1 = 35.0f64;
                    let d2 = 85.0f64;
                    let h = 950.0f64;
                    self.geo_volume = (std::f64::consts::PI * (d1 * d1 + d1 * d2 + d2 * d2) * h / 12.0) as f32;
                    self.geo_max_d = 85.0f32;
                    self.geo_segments = 50.0;
                    self.ui.label(cx, ids!(length_value)).set_text(cx, "950");
                    self.ui.label(cx, ids!(top_value)).set_text(cx, "35.0");
                    self.ui.label(cx, ids!(bell_value)).set_text(cx, "85.0");
                    self.ui.label(cx, ids!(geo_profile)).set_text(cx, "Profile: Cone");
                    self.ui.label(cx, ids!(segments_value)).set_text(cx, "50");
                }
1 => {
                     // Open Project — use file dialog
                     if let Some(path) = rfd::FileDialog::new()
                         .set_file_name("cadsd-project.json")
                         .add_filter("JSON", &["json"])
                         .pick_file()
                     {
                         if let Ok(json) = std::fs::read_to_string(&path) {
                             if let Ok(project) = serde_json::from_str::<ProjectState>(&json) {
                                 self.current_geo = Some(project.geo);
                                 self.bubbles = project.bubbles;
                                 self.geo_length = project.length;
                                 self.top_diameter = project.top;
                                 self.geo_bell = project.bell;
                                 self.bore_style_name = match project.style {
                                     1 => "Kigali".to_string(),
                                     2 => "Mbeya".to_string(),
                                     _ => "Cone".to_string(),
                                 };
                                 self.geo_segments = project.segments as f32;
                                 self.bubble_count = self.bubbles.len();
                                 self.push_current_state();
                                 self.ui.label(cx, ids!(length_value)).set_text(cx, &format!("{:.0}", self.geo_length));
                                 self.ui.label(cx, ids!(top_value)).set_text(cx, &format!("{:.1}", self.top_diameter));
                                 self.ui.label(cx, ids!(bell_value)).set_text(cx, &format!("{:.1}", self.geo_bell));
                                 self.ui.label(cx, ids!(geo_profile)).set_text(cx, &format!("Profile: {}", self.bore_style_name));
                                 self.ui.label(cx, ids!(segments_value)).set_text(cx, &format!("{}", self.geo_segments as usize));
                                 ::log::info!("Loaded project: {}", path.display());
                             } else {
                                 ::log::error!("Failed to parse project JSON: {}", path.display());
                             }
                         } else {
                             ::log::error!("Failed to read project file: {}", path.display());
                         }
                     }
                     // Reset dropdown selection to prevent menu glitch
                     self.ui.drop_down(cx, ids!(file_menu)).set_selected_item(cx, 0);
                 }
2 => {
                     // Save Project — use file dialog
                     if let Some(path) = rfd::FileDialog::new()
                         .set_file_name("cadsd-project.json")
                         .add_filter("JSON", &["json"])
                         .save_file()
                     {
                         use serde::Serialize;
                         #[derive(Serialize)]
                         struct ProjectSave {
                             geo: serde_json::Value,
                             bubbles: Vec<(f64, f64, f64)>,
                             length: f32,
                             top: f32,
                             bell: f32,
                             style: u32,
                             bore_curve: f32,
                             segments: usize,
                         }
                         if let Some(geo) = &self.current_geo {
                             let save = ProjectSave {
                                 geo: serde_json::json!(geo.geo),
                                 bubbles: self.bubbles.iter().map(|b| (b.0 as f64, b.1 as f64, b.2 as f64)).collect(),
                                 length: self.geo_length,
                                 top: self.top_diameter,
                                 bell: self.geo_bell,
                                 style: match self.bore_style_name.as_str() {
                                     "Kigali" => 1,
                                     "Mbeya" => 2,
                                     _ => 0,
                                 },
                                 bore_curve: self.geo_taper,
                                 segments: self.geo_segments as usize,
                             };
                             if let Ok(json) = serde_json::to_string_pretty(&save) {
                                 if let Err(e) = std::fs::write(&path, json) {
                                     ::log::error!("Failed to save project: {}", e);
                                 } else {
                                     ::log::info!("Saved project to: {}", path.display());
                                 }
                             }
                         }
                        
                        
                     }
                     // Reset dropdown selection to prevent menu glitch
                     self.ui.drop_down(cx, ids!(file_menu)).set_selected_item(cx, 0);
                 }
3 => {
                      // Save As... — save current project as JSON
                      if let Some(path) = rfd::FileDialog::new()
                          .set_file_name("cadsd-project.json")
                          .add_filter("JSON", &["json"])
                          .save_file()
                      {
                          use serde::Serialize;
                          #[derive(Serialize)]
                          struct ProjectSave {
                              geo: serde_json::Value,
                              bubbles: Vec<(f64, f64, f64)>,
                              length: f32,
                              top: f32,
                              bell: f32,
                              style: u32,
                              bore_curve: f32,
                              segments: usize,
                          }
                          if let Some(geo) = &self.current_geo {
                              let save = ProjectSave {
                                  geo: serde_json::json!(geo.geo),
                                  bubbles: self.bubbles.iter().map(|b| (b.0 as f64, b.1 as f64, b.2 as f64)).collect(),
                                  length: self.geo_length,
                                  top: self.top_diameter,
                                  bell: self.geo_bell,
                                  style: match self.bore_style_name.as_str() {
                                      "Kigali" => 1,
                                      "Mbeya" => 2,
                                      _ => 0,
                                  },
                                  bore_curve: self.geo_taper,
                                  segments: self.geo_segments as usize,
                              };
                              if let Ok(json) = serde_json::to_string_pretty(&save) {
                                  if let Err(e) = std::fs::write(&path, json) {
                                      ::log::error!("Failed to save project as: {}", e);
                                  } else {
                                      ::log::info!("Saved project as: {}", path.display());
                                  }
                              }
                          }
                          self.project_path = Some(path);
                      }
                      // Reset dropdown selection to prevent menu glitch
                      self.ui.drop_down(cx, ids!(file_menu)).set_selected_item(cx, 0);
                  }
4 => {
                     // Export — export current geometry and impedance data
                     self.export_impedance_csv();
                     self.export_geometry_json();
                     // Reset dropdown selection to prevent menu glitch
                     self.ui.drop_down(cx, ids!(file_menu)).set_selected_item(cx, 0);
                 }
                5 => {
                    // Quit
                    cx.quit();
                }
                _ => {}
            }
        }

        if let Some(idx) = self.ui.drop_down(cx, ids!(view_menu)).selected(actions) {
            match idx {
                0 => {
                    // Toggle Sidebar
                    self.show_sidebar = !self.show_sidebar;
                    self.ui.widget(cx, ids!(sidebar)).set_visible(cx, self.show_sidebar);
                }
                1 => {
                    // Toggle Wireframe
                    self.show_wireframe = !self.show_wireframe;
                    if let Some(mut vp) = self.ui.widget(cx, ids!(viewport)).borrow_mut::<BoreViewport>() {
                        vp.draw_mesh.wireframe = if self.show_wireframe { 1.0 } else { 0.0 };
                    }
                }
                2 => {
                    // Toggle Cross-Section
                    self.show_cross_section = !self.show_cross_section;
                    self.ui.widget(cx, ids!(cross_section)).set_visible(cx, self.show_cross_section);
                }
                3 => {
                    // Zoom Extents — reset camera to default orbit for current geometry
                    if let Some(mut vp) = self.ui.widget(cx, ids!(viewport)).borrow_mut::<BoreViewport>() {
                        vp.camera.orbit_yaw = 0.72;
                        vp.camera.orbit_pitch = -0.34;
                        vp.camera.distance = 70.0;
                    }
                }
                 4 => {
                    // Full Screen — toggle window fullscreen
                    let window = self.ui.window(cx, ids!(main_window));
                    if window.is_fullscreen(cx) {
                        window.disable_fullscreen(cx);
                    } else {
                        window.fullscreen(cx);
                    }
                }
                _ => {}
            }
        }

        // Handle Edit menu
        if let Some(idx) = self.ui.drop_down(cx, ids!(edit_menu)).selected(actions) {
            match idx {
                0 => {
                    // Undo
                    if self.history_index > 0 {
                        self.history_index -= 1;
                        self.restore_from_history(cx);
                    }
                }
                1 => {
                    // Redo
                    if self.history_index + 1 < self.history.len() {
                        self.history_index += 1;
                        self.restore_from_history(cx);
                    }
                }
2 => {
                     // Preferences — open preferences modal and sync controls
                     self.ui.modal(cx, ids!(preferences_modal)).open(cx);
                     // Sync controls with current state
                     let theme_idx = match self.theme_mode.as_str() {
                         "light" => 1,
                         "auto" => 2,
                         _ => 0,
                     };
self.ui.drop_down(cx, ids!(theme_dropdown)).set_selected_item(cx, theme_idx);
                      let backend_idx = match self.pref_simulation_backend.as_str() {
                          "waveguide" => 1,
                          "complex_impedance" => 2,
                          _ => 0,
                      };
                      self.ui.drop_down(cx, ids!(backend_dropdown)).set_selected_item(cx, backend_idx);
                      if let Some(mut check) = self.ui.widget(cx, ids!(wireframe_check)).borrow_mut::<CheckBox>() {
                          check.set_active(cx, self.show_wireframe, Animate::No);
                      }
                      if let Some(mut check) = self.ui.widget(cx, ids!(cross_section_check)).borrow_mut::<CheckBox>() {
                          check.set_active(cx, self.show_cross_section, Animate::No);
                      }
                      self.ui.slider(cx, ids!(volume_slider)).set_value(cx, self.pref_volume as f64);
                      self.ui.label(cx, ids!(volume_value)).set_text(cx, &format!("{:.0}%", self.pref_volume * 100.0));
                  }
                _ => {}
            }
        }

        if let Some(idx) = self.ui.drop_down(cx, ids!(theme_menu)).selected(actions) {
            match idx {
                0 => { self.theme_mode = "dark".to_string(); }
                1 => { self.theme_mode = "light".to_string(); }
                2 => { self.theme_mode = "auto".to_string(); }
                _ => {}
            }
            self.apply_theme(cx);
        }

        if let Some(idx) = self.ui.drop_down(cx, ids!(help_menu)).selected(actions) {
            match idx {
                0 => {
                    // Documentation — open documentation modal
                    self.ui.modal(cx, ids!(documentation_modal)).open(cx);
                }
                1 => {
                    // About — open about modal
                    self.ui.modal(cx, ids!(about_modal)).open(cx);
                }
                _ => {}
            }
        }

        // Preferences modal real-time control handlers
        if let Some(idx) = self.ui.drop_down(cx, ids!(theme_dropdown)).selected(actions) {
            match idx {
                0 => { self.theme_mode = "dark".to_string(); }
                1 => { self.theme_mode = "light".to_string(); }
                2 => { self.theme_mode = "auto".to_string(); }
                _ => {}
            }
            self.apply_theme(cx);
        }
        if let Some(v) = self.ui.slider(cx, ids!(volume_slider)).slided(actions) {
            self.pref_volume = v as f32;
            self.ui.label(cx, ids!(volume_value)).set_text(cx, &format!("{:.0}%", v * 100.0));
        }
        if let Some(checked) = self.ui.widget(cx, ids!(wireframe_check)).borrow::<CheckBox>().and_then(|c| c.changed(actions)) {
            self.show_wireframe = checked;
            if let Some(mut vp) = self.ui.widget(cx, ids!(viewport)).borrow_mut::<BoreViewport>() {
                vp.draw_mesh.wireframe = if checked { 1.0 } else { 0.0 };
            }
        }
        if let Some(checked) = self.ui.widget(cx, ids!(cross_section_check)).borrow::<CheckBox>().and_then(|c| c.changed(actions)) {
            self.show_cross_section = checked;
            self.ui.widget(cx, ids!(cross_section)).set_visible(cx, checked);
        }
        if let Some(idx) = self.ui.drop_down(cx, ids!(backend_dropdown)).selected(actions) {
            match idx {
                0 => { self.pref_simulation_backend = "tlm_python".to_string(); }
                1 => { self.pref_simulation_backend = "waveguide".to_string(); }
                2 => { self.pref_simulation_backend = "complex_impedance".to_string(); }
                _ => {}
            }
        }

        // Handle Preferences modal Apply button
        if self.ui.button(cx, ids!(apply_btn)).clicked(actions) {
            let idx = self.ui.drop_down(cx, ids!(theme_dropdown)).selected_item();
            match idx {
                0 => { self.theme_mode = "dark".to_string(); }
                1 => { self.theme_mode = "light".to_string(); }
                2 => { self.theme_mode = "auto".to_string(); }
                _ => {}
            }
            let idx = self.ui.drop_down(cx, ids!(backend_dropdown)).selected_item();
            match idx {
                0 => { self.pref_simulation_backend = "tlm_python".to_string(); }
                1 => { self.pref_simulation_backend = "waveguide".to_string(); }
                2 => { self.pref_simulation_backend = "complex_impedance".to_string(); }
                _ => {}
            }
            self.apply_theme(cx);
            self.ui.modal(cx, ids!(preferences_modal)).close(cx);
        }

        // Handle Preferences modal Cancel button
        if self.ui.button(cx, ids!(cancel_btn)).clicked(actions) {
            self.ui.modal(cx, ids!(preferences_modal)).close(cx);
        }

        // Handle Documentation modal buttons
        if self.ui.button(cx, ids!(ui_requirements_btn)).clicked(actions) {
            let _ = open::that("README.md");
        }
        if self.ui.button(cx, ids!(gui_roadmap_btn)).clicked(actions) {
            let _ = open::that("GUI_ROADMAP.md");
        }
        if self.ui.button(cx, ids!(todo_btn)).clicked(actions) {
            let _ = open::that("TODO.md");
        }
        if self.ui.button(cx, ids!(research_btn)).clicked(actions) {
            let _ = open::that("docs/RESEARCH.md");
        }

        // Handle modal dismissal
        if self.ui.modal(cx, ids!(preferences_modal)).dismissed(actions) {
            self.ui.modal(cx, ids!(preferences_modal)).close(cx);
        }
        if self.ui.modal(cx, ids!(about_modal)).dismissed(actions) {
            self.ui.modal(cx, ids!(about_modal)).close(cx);
        }
        if self.ui.modal(cx, ids!(documentation_modal)).dismissed(actions) {
            self.ui.modal(cx, ids!(documentation_modal)).close(cx);
        }
        if self.ui.modal(cx, ids!(evolution_modal)).dismissed(actions) {
            self.ui.modal(cx, ids!(evolution_modal)).close(cx);
        }

        // Handle view selector dropdown
        if let Some(idx) = self.ui.drop_down(cx, ids!(current_view_selector)).selected(actions) {
            let views = ["Setup", "Segments", "Bubbles", "Optimization", "Export"];
            self.current_view = views[idx].to_string();

            // Show viewport always
            self.ui.widget(cx, ids!(viewport)).set_visible(cx, true);

            match idx {
                0 => {
                    // Setup: all panels
                    self.ui.widget(cx, ids!(impedance_preview)).set_visible(cx, true);
                    self.ui.widget(cx, ids!(cross_section)).set_visible(cx, self.show_cross_section);
                    self.ui.widget(cx, ids!(geometry_summary)).set_visible(cx, true);
                    self.ui.widget(cx, ids!(resonance_analysis)).set_visible(cx, true);
                }
                1 => {
                    // Segments: viewport + geometry summary + resonance analysis
                    self.ui.widget(cx, ids!(impedance_preview)).set_visible(cx, true);
                    self.ui.widget(cx, ids!(cross_section)).set_visible(cx, self.show_cross_section);
                    self.ui.widget(cx, ids!(geometry_summary)).set_visible(cx, true);
                    self.ui.widget(cx, ids!(resonance_analysis)).set_visible(cx, true);
                }
                2 => {
                    // Bubbles: viewport only
                    self.ui.widget(cx, ids!(impedance_preview)).set_visible(cx, false);
                    self.ui.widget(cx, ids!(cross_section)).set_visible(cx, false);
                    self.ui.widget(cx, ids!(geometry_summary)).set_visible(cx, false);
                    self.ui.widget(cx, ids!(resonance_analysis)).set_visible(cx, false);
                }
                3 => {
                    // Optimization: viewport + impedance preview
                    self.ui.widget(cx, ids!(impedance_preview)).set_visible(cx, true);
                    self.ui.widget(cx, ids!(cross_section)).set_visible(cx, false);
                    self.ui.widget(cx, ids!(geometry_summary)).set_visible(cx, false);
                    self.ui.widget(cx, ids!(resonance_analysis)).set_visible(cx, false);
                }
                4 => {
                    // Export: viewport only
                    self.ui.widget(cx, ids!(impedance_preview)).set_visible(cx, false);
                    self.ui.widget(cx, ids!(cross_section)).set_visible(cx, false);
                    self.ui.widget(cx, ids!(geometry_summary)).set_visible(cx, false);
                    self.ui.widget(cx, ids!(resonance_analysis)).set_visible(cx, false);
                }
                _ => {}
            }
        }
    }
}

impl App {
    fn apply_theme(&mut self, cx: &mut Cx) {
        let (text, muted) = match self.theme_mode.as_str() {
            "light" => (
                Vec4f { x: 0.102, y: 0.141, z: 0.188, w: 1.0 },
                Vec4f { x: 0.353, y: 0.420, z: 0.486, w: 1.0 },
            ),
            _ => (
                Vec4f { x: 0.871, y: 0.906, z: 0.933, w: 1.0 },
                Vec4f { x: 0.514, y: 0.569, z: 0.627, w: 1.0 },
            ),
        };

        if let Some(mut title) = self.ui.widget(cx, ids!(title)).borrow_mut::<Label>() {
            title.set_text_color(cx, text);
        }
        if let Some(mut hint) = self.ui.widget(cx, ids!(hint)).borrow_mut::<Label>() {
            hint.set_text_color(cx, muted);
        }
        if let Some(mut running_label) = self.ui.widget(cx, ids!(running_label)).borrow_mut::<Label>() {
            running_label.set_text_color(cx, muted);
        }
    }

    fn export_impedance_csv(&mut self) {
        if self.impedance_data.is_empty() {
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("impedance_spectrum.csv")
            .add_filter("CSV", &["csv"])
            .save_file()
        {
            let mut content = String::new();
            content.push_str("frequency_hz,impedance_magnitude\n");
            for point in &self.impedance_data {
                content.push_str(&format!("{},{}\n", point.x, point.y));
            }

            if let Err(e) = std::fs::write(&path, content) {
                ::log::error!("Failed to export CSV: {}", e);
            } else {
                ::log::info!("Exported impedance data to: {}", path.display());
            }
        }
    }

    fn export_geometry_json(&mut self) {
        if let Some(geo) = &self.current_geo {
            use serde::Serialize;

            #[derive(Serialize)]
            struct GeometryExport {
                segments: Vec<[f64; 2]>,
                bubbles: Vec<(f64, f64, f64)>,
                metadata: serde_json::Value,
            }

            let export = GeometryExport {
                segments: geo.geo.clone(),
                bubbles: self
                    .bubbles
                    .iter()
                    .map(|(pos, width, height)| (*pos as f64, *width as f64, *height as f64))
                    .collect(),
                metadata: serde_json::json!({
                    "length_mm": self.geo_length,
                    "top_diameter_mm": self.top_diameter,
                    "bell_diameter_mm": self.geo_bell,
                    "segments": self.geo_segments,
                    "bore_style": self.bore_style_name,
                    "bore_curve": self.geo_taper,
                }),
            };

            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("geometry.json")
                .add_filter("JSON", &["json"])
                .save_file()
            {
                if let Ok(json) = serde_json::to_string_pretty(&export) {
                    if let Err(e) = std::fs::write(&path, json) {
                        ::log::error!("Failed to export JSON: {}", e);
                    } else {
                        ::log::info!("Exported geometry to: {}", path.display());
                    }
                }
            }
        }
    }

    fn push_current_state(&mut self) {
        let geo = self.current_geo.clone().unwrap_or_else(|| create_base_geo(950.0, 35.0, 85.0, 0, 0.0, 50));
        let state = ProjectState {
            geo: geo.copy(),
            bubbles: self.bubbles.clone(),
            length: self.geo_length,
            top: self.top_diameter,
            bell: self.geo_bell,
            style: match self.bore_style_name.as_str() {
                "Kigali" => 1,
                "Mbeya" => 2,
                _ => 0,
            },
            bore_curve: 1.0,
            segments: self.geo_segments as usize,
        };
        if self.history.len() > self.history_index {
            self.history.truncate(self.history_index);
        }
        self.history.push(state);
        self.history_index = self.history.len();
    }

    fn restore_from_history(&mut self, cx: &mut Cx) {
        if self.history_index >= self.history.len() {
            return;
        }
        let state = &self.history[self.history_index];
        self.current_geo = Some(state.geo.copy());
        self.bubbles = state.bubbles.clone();
        self.geo_length = state.length;
        self.top_diameter = state.top;
        self.geo_bell = state.bell;
        self.geo_segments = state.segments as f32;
        self.bubble_count = self.bubbles.len();
        self.bore_style_name = match state.style {
            1 => "Kigali".to_string(),
            2 => "Mbeya".to_string(),
            _ => "Cone".to_string(),
        };
        if let Some(mut vp) = self.ui.widget(cx, ids!(viewport)).borrow_mut::<BoreViewport>() {
            vp.update_bore_geo(cx, self.current_geo.as_ref().unwrap());
        }
        self.ui.label(cx, ids!(length_value)).set_text(cx, &format!("{:.0}", self.geo_length));
        self.ui.label(cx, ids!(top_value)).set_text(cx, &format!("{:.1}", self.top_diameter));
        self.ui.label(cx, ids!(bell_value)).set_text(cx, &format!("{:.1}", self.geo_bell));
        self.ui.label(cx, ids!(segments_value)).set_text(cx, &format!("{}", self.geo_segments));
        self.ui.label(cx, ids!(geo_profile)).set_text(cx, &format!("Profile: {}", self.bore_style_name));
    }

    

    fn update_loss_values(&mut self) {
        if let Some(ref geo) = self.current_geo {
            let freqs = get_log_simulation_frequencies();
            let impedances = match acoustical_simulation(&geo, &freqs, "tlm_python") {
                Ok(impedances) => impedances,
                Err(_) => {
                    self.tairua_loss_value = 0.0;
                    self.loss_fundamental_value = 0.0;
                    self.loss_harmonics_value = 0.0;
                    self.loss_peaks_value = 0.0;
                    return;
                }
            };

            let mut total_loss = 0.0;

            let mut peaks = Vec::new();
            let mag: Vec<f64> = impedances.iter().map(|c| c.abs()).collect();
            for i in 1..mag.len() - 1 {
                if mag[i] > mag[i - 1] && mag[i] > mag[i + 1] {
                    peaks.push((i, freqs[i], mag[i]));
                }
            }

            if !peaks.is_empty() {
                if let Some(first_peak) = peaks.first() {
                    let f0 = first_peak.1;
                    let freq_diff = (f0 - self.target_freq).abs();
                    let fundamental_loss = (freq_diff / 5.0).powi(2);
                    self.loss_fundamental_value = fundamental_loss * self.weight_fundamental;
                }

                if peaks.len() >= 2 && self.target_freq > 0.0 {
                    let f0 = peaks[0].1;
                    let mut harmonic_loss = 0.0;
                    for (i, (_, freq, _)) in peaks.iter().enumerate().skip(1) {
                        let expected_harmonic = f0 * (i + 1) as f64;
                        let harmonic_deviation =
                            (freq - expected_harmonic).abs() / expected_harmonic;
                        harmonic_loss += harmonic_deviation;
                    }
                    harmonic_loss /= peaks.len() as f64;
                    self.loss_harmonics_value = harmonic_loss * self.weight_harmonics;
                }

                if !peaks.is_empty() {
                    let peak_count_loss =
                        (5usize as f64 - peaks.len().min(5) as f64).max(0.0) / 5.0;
                    let avg_mag =
                        peaks.iter().map(|(_, _, mag)| *mag).sum::<f64>() / peaks.len() as f64;
                    let mag_loss = (1.0 - avg_mag).max(0.0);
                    self.loss_peaks_value = (peak_count_loss + mag_loss) * self.weight_peaks;
                }

                total_loss =
                    self.loss_fundamental_value + self.loss_harmonics_value + self.loss_peaks_value;
            }

            self.tairua_loss_value = total_loss.min(10.0);

            self.loss_total_text = if total_loss > 0.0 {
                format!("Total Tairua Loss: {:.3}", total_loss)
            } else {
                "Total Tairua Loss: -".to_string()
            };
            self.loss_fundamental_text = if self.loss_fundamental_value > 0.0 {
                format!(
                    "Fundamental Loss: {:.3} (w: {:.1})",
                    self.loss_fundamental_value, self.weight_fundamental
                )
            } else {
                "Fundamental Loss: -".to_string()
            };
            self.loss_harmonics_text = if self.loss_harmonics_value > 0.0 {
                format!(
                    "Harmonic Loss: {:.3} (w: {:.1})",
                    self.loss_harmonics_value, self.weight_harmonics
                )
            } else {
                "Harmonic Loss: -".to_string()
            };
            self.loss_peaks_text = if self.loss_peaks_value > 0.0 {
                format!(
                    "Peak Loss: {:.3} (w: {:.1})",
                    self.loss_peaks_value, self.weight_peaks
                )
            } else {
                "Peak Loss: -".to_string()
            };
        }
    }

    fn update_cross_section_data(&mut self) {
        if let Some(ref geo) = self.current_geo {
            let mut data = Vec::new();
            for (_i, segment) in geo.geo.iter().enumerate() {
                let x = segment[0] as f32;
                let radius = segment[1] as f32;
                data.push((x, radius));
            }
            self.cross_section_data = data;
        } else {
            self.cross_section_data = vec![];
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
