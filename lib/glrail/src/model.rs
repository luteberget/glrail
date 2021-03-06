use crate::infrastructure::*;
use crate::dgraph::*;
use crate::schematic::*;
use crate::interlocking::*;
use crate::view::*;
use crate::issue::*;
use crate::selection::*;
use crate::vehicle::*;
use crate::scenario::*;
use serde::{Serialize, Deserialize};



#[derive(Serialize, Deserialize)]
pub enum Derive<T> {
    Wait,
    Ok(T),
    Err(String),
}

impl<T :Default> Default for Derive<T> {
    fn default() -> Self {
        Derive::Ok(Default::default())
    }
}

impl<T> Derive<T> {
    pub fn get(&self) -> Option<&T> {
        if let Derive::Ok(val) = self { Some(val) } else { None }
    }
}

pub enum ModelUpdateResult {
    NoChange,
    InfrastructureChanged,
    //SchematicChanged,
    //ViewChanged,
    InterlockingChanged,
    ScenarioChanged(usize),
}

pub enum ModelAction {
    Inf(InfrastructureEdit),
    //Interlocking(InterlockingEdit),
    Scenario(ScenarioEdit),
}

#[derive(Serialize, Deserialize)]
pub struct Model {
    pub inf :Infrastructure,
    pub schematic :Derive<Schematic>,
    pub view :View,
    pub interlocking: Interlocking,
    pub scenarios :Vec<Scenario>,
    pub vehicles :Vec<Vehicle>,

    #[serde(skip)]
    pub dgraph: Derive<DGraph>,
}


impl Model {

    //pub fn selected_entity(&mut self) -> Option<(EntityId, &Entity)> {
    //    let id = if let Selection::Entity(id) = self.view.selection { Some(id) } else { None } ?;
    //    self.inf.entities.get(id)?.as_ref().map(|e| (id,e))
    //}

    pub fn new_empty() -> Self {
        Model {
            inf: Infrastructure::new_empty(),
            schematic: Derive::Ok(Schematic::new_empty()),
            view: View::new_default(),
            interlocking: Interlocking::new_default(),
            dgraph: Default::default(),
            scenarios : Vec::new(),
            vehicles: vec![default_vehicle()],
        }
    }

    pub fn select_pos(&mut self, pos :f32, obj :TrackId) {
        let y = 0.0;
        self.view.selection = Selection::Pos(pos, y, obj );
        //println!("select pos {:?}", self.view.selection);
    }

    pub fn integrate(&mut self, action :ModelAction) -> ModelUpdateResult {
        match self.handle_event(action) {
            Ok(r) => r,
            Err(s) => {
                println!("ERROR: {:?}", s);
                ModelUpdateResult::NoChange
            },
        }
    }

    pub fn iter_issues(&self) -> impl Iterator<Item = Issue> {
        use std::iter;
        iter::empty()
    }


        pub fn move_view(&mut self, inputdir: InputDir) {
        match inputdir {
            InputDir::Left => (self.view.viewport.0).0 -= 0.15*self.view.viewport.1,
            InputDir::Right => (self.view.viewport.0).0 += 0.15*self.view.viewport.1,
            InputDir::Up => (self.view.viewport.0).1 += 0.15*self.view.viewport.1,
            InputDir::Down => (self.view.viewport.0).1 -= 0.15*self.view.viewport.1,
        }
    }

    pub fn include_in_view(&mut self, pt: (f32,f32))  {
        //unimplemented!()
    }

    pub fn entity_location(&self, obj :EntityId) -> (f32,f32) {
        return (0.0,0.0);
        //unimplemented!()
    }

    pub fn move_selection(&mut self, inputdir: InputDir) {
        //   println!("move selection");
        //   match &self.view.selection {
        //       Selection::None => { 
        //           if let Some(id) = self.inf.any_object() {
        //               self.view.selection = Selection::Entity(id);
        //               self.include_in_view(self.entity_location(id));
        //           }
        //   println!("move selection: none");
        //       },
        //       Selection::Entity(i) => {
        //           //if let Some(Some(Entity::Node(_, n))) = self.inf.entities.get(*i) {
        //           //    for p in app.inf.node_ports(i) {
        //           //        match (n,p) {
        //           //            (Node::BufferStop, Port::Out) => {
        //           //                // ...
        //           //            },
        //           //        }
        //           //    }
        //           //}
        //       },
        //       Selection::Pos(pos, y, track_id) => {
        //   println!("move selection: pos");
        //           if let Some(Some(Entity::Track(Track { start_node, end_node, ..}))) = self.inf.entities.get(*track_id) {
        //               match inputdir {
        //                   InputDir::Right => { 
        //                       self.view.selection = Selection::Entity(Entity::NodeId(end_node.0));
        //                       self.include_in_view(self.entity_location(Entity::NodeId(end_node.0)));
        //                   },
        //                   InputDir::Left => { 
        //                       self.view.selection = Selection::Entity(Entity::NodeId(start_node.0));
        //                       self.include_in_view(self.entity_location(Entity::NodeId(start_node.0)));
        //                   },
        //                   _ => {},
        //               }
        //           }
        //       },
        //       _ => { unimplemented!() },
        //   }
    }

    pub fn handle_event(&mut self, action :ModelAction) -> Result<ModelUpdateResult, String> {
        match action {
            ModelAction::Inf(ie) => {
                match ie {
                    InfrastructureEdit::Invalidate => {},
                    InfrastructureEdit::NewTrack(p1,p2) => {
                        let inf = &mut self.inf;
                        let i1 = self.inf.new_node(Node(p1, NodeType::Macro(None)));
                        let i2 = self.inf.new_node(Node(p2, NodeType::Macro(None)));
                        let t =  self.inf.new_track(Track {
                            start_node: (i1, Port { dir: Dir::Up, course: None }),
                            end_node:   (i2, Port { dir: Dir::Down, course: None }),
                        });
                    },
                    InfrastructureEdit::InsertObject(t,p,obj) => {
                        let _id = self.inf.new_object(Object(t,p,obj));
                    },
                    InfrastructureEdit::ToggleBufferMacro(node_id) => {
                        if let Some(ref mut node) = self.inf.get_node_mut(&node_id) {
                            if let NodeType::BufferStop = node.1 {
                                node.1 = NodeType::Macro(None);
                            } else if let NodeType::Macro(_) = node.1 {
                                node.1 = NodeType::BufferStop;
                            }
                        }
                    }
                    InfrastructureEdit::InsertNode(track_id,p,node,l) => { // TrackId, Pos, NodeType, f32
                        let (straight_side, branch_side) = match node {
                            NodeType::Switch(_,side) => (side.other(), side),
                            _ => unimplemented!(),
                        };
                        let new = self.inf.new_node(Node(p, node.clone()));
                        let inf = &mut self.inf;

                        let t = inf.get_track_mut(&track_id).ok_or("Track ref err".to_string())?;

                        match &node {
                            NodeType::Switch(Dir::Up, _) => {
                                let old_end = t.end_node;

                                t.end_node = (new, Port { dir: Dir::Down, course: None });

                                let _straight = self.inf.new_track(Track {
                                    start_node: (new, Port { dir: Dir::Up, course: Some(straight_side) }),
                                    end_node: old_end,
                                });

                                let branch_end = self.inf.new_node(Node(p+l, NodeType::BufferStop));
                                let branch = self.inf.new_track(Track {
                                    start_node: (new, Port { dir: Dir::Up, course: Some(branch_side) }),
                                    end_node: (branch_end, Port { dir: Dir::Down, course: None }),
                                });
                            },
                            NodeType::Switch(Dir::Down, _) => {
                                let old_start = t.start_node;
                                t.start_node = (new, Port { dir: Dir::Up, course: None });

                                let _straight = self.inf.new_track(Track {
                                    start_node: old_start,
                                    end_node:   (new, Port { dir: Dir::Down, course: Some(straight_side) })
                                });

                                let branch_start = self.inf.new_node(Node(p-l, NodeType::BufferStop));
                                let branch = self.inf.new_track(Track {
                                    start_node: (branch_start, Port { dir: Dir::Up, course: None }),
                                    end_node:   (new, Port { dir: Dir::Down, course: Some(branch_side) }),
                                });
                            },
                            _ => unimplemented!()
                        };

                        self.view.selection = Selection::Entity(EntityId::Node(new));

                    },
                    InfrastructureEdit::JoinNodes(n1,n2) => {
                        let inf = &mut self.inf;
                        let Node(_,n1_obj) = inf.get_node(&n1).ok_or("Node ref err".to_string())?;
                        let Node(_,n2_obj) = inf.get_node(&n2).ok_or("Node ref err".to_string())?;

                        if n1_obj.num_ports() != 1 || n2_obj.num_ports() != 1 {
                            return Err("Nodes must have 1 port.".to_string());
                        }

                        let mut lo_track = None;
                        let mut hi_track = None;

                        for (track_id, track) in inf.iter_tracks() {
                            if track.start_node.0 == n1 { hi_track = Some((track_id,n1)); }
                            if track.start_node.0 == n2 { hi_track = Some((track_id,n2)); }
                            if track.end_node.0 == n1   { lo_track = Some((track_id,n1)); }
                            if track.end_node.0 == n2   { lo_track = Some((track_id,n2)); }
                        }

                        match (lo_track,hi_track) {
                            (Some((t1,n1)),Some((t2,n2))) => {
                                let end_node = inf.get_track_mut(&t2).unwrap().end_node;
                                let track1 = inf.get_track_mut(&t1).unwrap();
                                track1.end_node = end_node;
                                inf.delete(EntityId::Track(t2));
                                inf.delete(EntityId::Node(n1));
                                inf.delete(EntityId::Node(n2));
                            },
                            _ => return Err("Mismatching nodes for joining".to_string())
                        }

                    },
                    InfrastructureEdit::ExtendTrack(node_id, length) => {
                        let inf = &mut self.inf;
                        if let Some(Node(ref mut node_pos,_)) = inf.get_node_mut(&node_id) {
                            *node_pos += length;
                        }
                    },
                };
                Ok(ModelUpdateResult::InfrastructureChanged)
            },
            ModelAction::Scenario(se) => {
                match se {
                    // DISPATCH
                    ScenarioEdit::NewDispatch => {
                        let i = self.scenarios.len();
                        self.scenarios.push(Scenario::Dispatch(Default::default()));
                        Ok(ModelUpdateResult::ScenarioChanged(i))
                    },
                    ScenarioEdit::AddDispatchCommand(i,time,cmd) => {
                        if let Some(Scenario::Dispatch(Dispatch { commands, ..  })) = self.scenarios.get_mut(i) {
                            commands.push((time,cmd));
                        }
                        Ok(ModelUpdateResult::ScenarioChanged(i))
                    },
                    ScenarioEdit::ModifyDispatchCommand(i, cmd_idx, new) => {
                        Ok(ModelUpdateResult::ScenarioChanged(i))
                    },

                    // USAGE
                    ScenarioEdit::NewUsage => {
                        let i = self.scenarios.len();
                        self.scenarios.push(Scenario::Usage(Default::default(), Default::default()));
                        Ok(ModelUpdateResult::ScenarioChanged(i))
                    },
                    ScenarioEdit::AddUsageMovement(si) => {
                        if let Some(Scenario::Usage(ref mut usage, _)) = self.scenarios.get_mut(si) {
                            usage.movements.push(Default::default());
                        }
                        Ok(ModelUpdateResult::ScenarioChanged(si))
                    },
                    ScenarioEdit::SetUsageMovementVehicle(si,mi,vi) => {
                        if let Some(Scenario::Usage(ref mut usage, _)) = self.scenarios.get_mut(si) {
                            if let Some(movement) = usage.movements.get_mut(mi) {
                                movement.vehicle_ref = vi;
                            }
                        }
                        Ok(ModelUpdateResult::ScenarioChanged(si))
                    },
                    ScenarioEdit::AddUsageMovementVisit(si,mi) => {
                        if let Some(Scenario::Usage(ref mut usage, _)) = self.scenarios.get_mut(si) {
                            if let Some(movement) = usage.movements.get_mut(mi) {
                                movement.visits.push(Default::default());
                            }
                        }
                        Ok(ModelUpdateResult::ScenarioChanged(si))
                    },

                    ScenarioEdit::SetUsageMovementVisitNodes(si,mi,vi,nodes) => {
                        if let Some(Scenario::Usage(ref mut usage, _)) = self.scenarios.get_mut(si) {
                            if let Some(movement) = usage.movements.get_mut(mi) {
                                if let Some(visit) = movement.visits.get_mut(vi) {
                                    visit.nodes = nodes;
                                }
                            }
                        }
                        Ok(ModelUpdateResult::ScenarioChanged(si))
                    },
                    ScenarioEdit::AddUsageTimingSpec(si) => {
                        if let Some(Scenario::Usage(ref mut usage, _)) = self.scenarios.get_mut(si) {
                            usage.timings.push(TimingSpec { visit_a : (0,0), visit_b : (0,0), time : None });
                        }
                        Ok(ModelUpdateResult::ScenarioChanged(si))
                    },
                    ScenarioEdit::SetUsageTimingSpec(si,ti,am,av,bm,bv,time) => {
                        if let Some(Scenario::Usage(ref mut usage, _)) = self.scenarios.get_mut(si) {
                            if let Some(spec) = usage.timings.get_mut(ti) {
                                spec.visit_a = (am,av);
                                spec.visit_b = (bm,bv);
                                spec.time = time;
                            }
                        }
                        Ok(ModelUpdateResult::ScenarioChanged(si))
                    },
                }
            }
        }
    }


}



