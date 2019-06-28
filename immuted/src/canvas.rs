use crate::model::*;
use std::collections::HashSet;
use crate::ui;
use crate::util;
use crate::view::*;
use crate::ui::col;
use crate::ui::ImVec2;
use backend_glfw::imgui::*;
use nalgebra_glm as glm;
use const_cstr::const_cstr;

pub struct Canvas {
    action :Action,
    selection :HashSet<Ref>,
    view :View,
}

#[derive(Debug)]
pub enum Action {
    Normal(NormalState),
    DrawingLine(Option<Pt>),
    DrawObjectType(Option<usize>),
}

#[derive(Debug,Copy,Clone)]
pub enum NormalState {
    Default,
    SelectWindow(ImVec2),
    DragMove,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            action :Action::Normal(NormalState::Default),
            selection :HashSet::new(),
            view :View::default(),
        }
    }

    pub fn toolbar(&mut self, doc :&mut Undoable<Model>) {

    }

    pub fn draw(&mut self, doc :&mut Undoable<Model>, size :ImVec2) {
        let zero = ImVec2 { x: 0.0, y: 0.0 };
        use backend_glfw::imgui::*;
        ui::canvas(size, |draw_list, pos| { unsafe {

            // Hotkeys
            self.handle_global_keys(doc);
            let handle_keys = igIsItemActive() || !igIsAnyItemActive();
            if handle_keys { self.handle_keys(); }

            // Scroll action (wheel or ctrl-drag)
            self.scroll();

            let io = igGetIO();
            let pointer = (*io).MousePos - pos;
            let pointer_ongrid = self.view.screen_to_world_pt(pointer);
            let pointer_ingrid = self.view.screen_to_world_ptc(pointer);

            // Context menu 
            if igBeginPopup(const_cstr!("ctx").as_ptr(), 0 as _) {
                igText(const_cstr!("Selection?").as_ptr());

                igEndPopup();
            }

            // Edit actions 
            match &self.action {
                Action::Normal(normal) => {
                    let normal = *normal;
                    self.normalstate(normal, doc, draw_list, pointer_ingrid, pos);
                }
                Action::DrawingLine(from) => {
                    let from = *from;
                    self.drawingline(doc,from,pos,pointer_ongrid,draw_list);
                }
                _ => panic!(), // TODO
            };

            // Draw background
            self.draw_background(doc.get(), draw_list, pos, size);


        }});
    }

    pub fn handle_keys(&mut self) {
        unsafe {
        if igIsKeyPressed('A' as _, false) {
            self.action = Action::Normal(NormalState::Default);
        }
        if igIsKeyPressed('D' as _, false) {
            self.action = Action::DrawingLine(None);
        }
        if igIsKeyPressed('S' as _, false) {
            self.action = Action::DrawObjectType(None);
        }
        }
    }

    pub fn handle_global_keys(&mut self, doc :&mut Undoable<Model>) { unsafe {
        let io = igGetIO();
        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('Z' as _, false) {
            doc.undo();
        }
        if (*io).KeyCtrl && (*io).KeyShift && igIsKeyPressed('Z' as _, false) {
            doc.redo();
        }
        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('Y' as _, false) {
            doc.redo();
        }
    } }

    pub fn scroll(&mut self) {
        unsafe {
            let io = igGetIO();
            let wheel = (*io).MouseWheel;
            if wheel != 0.0 {
                self.view.zoom(wheel);
            }
            if ((*io).KeyCtrl && igIsMouseDragging(0,-1.0)) || igIsMouseDragging(2,-1.0) {
                self.view.translate((*io).MouseDelta);
            }
        }
    }

    pub fn draw_background(&self, m :&Model, draw_list :*mut ImDrawList, pos :ImVec2, size :ImVec2) {
        unsafe {

            let sel_window = if let Action::Normal(NormalState::SelectWindow(a)) = &self.action {
                Some((*a, *a + igGetMouseDragDelta_nonUDT2(0,-1.0).into()))
            } else { None };

            for l in &m.linesegs {
                let p1 = self.view.world_pt_to_screen(l.0);
                let p2 = self.view.world_pt_to_screen(l.1);
                let selected = self.selection.contains(&Ref::Track(l.0,l.1));
                let preview = sel_window
                    .map(|(a,b)| util::point_in_rect(p1,a,b) || util::point_in_rect(p2,a,b))
                    .unwrap_or(false) ;
                let col = if selected || preview { col::selected() } else { col::unselected() };
                ImDrawList_AddLine(draw_list, pos + p1, pos + p2, col, 2.0);
            }

            let (lo,hi) = self.view.points_in_view(size);
            for x in lo.x..=hi.x {
                for y in lo.y..=hi.y {
                    let pt = self.view.world_pt_to_screen(glm::vec2(x,y));
                    ImDrawList_AddCircleFilled(draw_list, pos+pt, 3.0, col::gridpoint(), 4);
                }
            }
        }
    }

    pub fn set_selection_window(&mut self, a :ImVec2, b :ImVec2) {
        // 
    }

    pub fn normalstate(&mut self, state: NormalState, doc :&Undoable<Model>,
                       draw_list :*mut ImDrawList, pointer_ingrid :PtC, pos :ImVec2) {
        unsafe {
        let io = igGetIO();
        match state {
            NormalState::SelectWindow(a) => {
                let b = a + igGetMouseDragDelta_nonUDT2(0,-1.0).into();
                if igIsMouseDragging(0,-1.0) {
                    ImDrawList_AddRect(draw_list, pos + a, pos + b,
                                       col::selected(),0.0, 0, 1.0);
                } else {
                    self.set_selection_window(a,b);
                    self.action = Action::Normal(NormalState::Default);
                }
            },
            NormalState::DragMove => {
                if igIsMouseDragging(0,-1.0) {
                    println!("Dragging {:?}", self.selection);
                    // TODO 
                } else {
                    self.action = Action::Normal(NormalState::Default);
                }
            }
            NormalState::Default => {
                if igIsMouseDragging(0,-1.0) {
                    if let Some(r) = doc.get().get_closest(pointer_ingrid) {
                        if !self.selection.contains(&r) {
                            self.selection = std::iter::once(r).collect();
                        }
                        self.action = Action::Normal(NormalState::DragMove);
                    } else {
                        let a = (*io).MouseClickedPos[0] - pos;
                        //let b = a + igGetMouseDragDelta_nonUDT2(0,-1.0).into();
                        self.action = Action::Normal(NormalState::SelectWindow(a));
                    }
                } else {
                    if igIsMouseReleased(0) {
                        if !(*io).KeyShift { self.selection.clear(); }
                        if let Some(r) = doc.get().get_closest(pointer_ingrid) {
                            self.selection.insert(r);
                        } 
                    }
                    if igIsMouseClicked(1,false) {
                        igOpenPopup(const_cstr!("ctx").as_ptr());
                    }
                }
            },
        }
        }
    }

    pub fn drawingline(&mut self,  doc :&mut Undoable<Model>,from :Option<Pt>,
                       pos :ImVec2, pointer_ongrid :Pt, draw_list :*mut ImDrawList
                       ) {
        unsafe {
        // Draw preview
        if let Some(pt) = from {
            for (p1,p2) in util::route_line(pt, pointer_ongrid) {
                ImDrawList_AddLine(draw_list, pos + self.view.world_pt_to_screen(p1),
                                              pos + self.view.world_pt_to_screen(p2), 
                                              col::selected(), 2.0);
            }

            if !igIsMouseDown(0) {
                let mut new_model = doc.get().clone();
                for (p1,p2) in util::route_line(pt,pointer_ongrid) {
                    let unit = util::unit_step_diag_line(p1,p2);
                    for (pa,pb) in unit.iter().zip(unit.iter().skip(1)) {
                        new_model.linesegs.insert(util::order_ivec(*pa,*pb));
                    }
                }
                doc.set(new_model);
                self.action = Action::DrawingLine(None);
            }
        } else {
            if igIsItemHovered(0) && igIsMouseDown(0) {
                self.action = Action::DrawingLine(Some(pointer_ongrid));
            }
        }
    } }
}

