use makepad_widgets::*;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;

    StripButton = <Button> {
        draw_bg: {
            instance left_radius: 0.0;
            instance right_radius: 0.0;
            instance step: 1.0;
            instance step_influence: 0.1;

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                // idk why 0.5 must be the minimum nor idk why 0.25 and not 0.5 for the multiplier
                let rl = mix(0.5, self.rect_size.y * 0.25, self.left_radius);
                let rr = mix(0.5, self.rect_size.y * 0.25, self.right_radius);

                // base color
                let fill_color = #15859A;

                // make the base color ligther as step grows
                let fill_color = mix(fill_color, #fff, self.step * self.step_influence);

                // make a gradiant over the x axis
                // use the step_infuence so this gradiant can be continued by the next button
                let fill_color = mix(fill_color, #fff, self.pos.x * self.step_influence);

                // make the color a little bit ligther when hovered
                let fill_color = mix(fill_color, #fff, self.hover * 0.3);

                sdf.box_all(0.0, 0.0, self.rect_size.x, self.rect_size.y, rl, rr, rr, rl);

                sdf.fill_keep(fill_color);

                return sdf.result;
            }
        },
    }

    SizedStripButton = <StripButton> {
        width: 100.0,
        height: 32.0,
    }

    Ui = {{Ui}} {
        align: {x: 0.5, y: 0.5}
        show_bg: true,
        draw_bg: {
            color: #fff
        }
        body = <RoundedView> {
            width: Fit,
            height: Fit,
            padding: 2;
            spacing: 2;
            draw_bg: {
                radius: 9.0;
                color: #fff;
                
            }
            <SizedStripButton> {
                draw_bg: {
                    instance step: 1.0;
                    left_radius: 1.0;
                }
            }
            <SizedStripButton> {
                draw_bg: {
                    instance step: 2.0;
                }
            }
            <SizedStripButton> {
                draw_bg: {
                    instance step: 3.0;
                }
            }
            <SizedStripButton> {
                draw_bg: {
                    instance step: 4.0;
                }
            }
            <SizedStripButton> {
                draw_bg: {
                    instance step: 5.0;
                    right_radius: 1.0;
                }
            }
        }
    }
);

#[derive(Live, LiveHook, Widget)]
pub struct Ui {
    #[deref]
    deref: Window,
}

impl Widget for Ui {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}
