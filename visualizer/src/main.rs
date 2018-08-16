#![feature(use_extern_macros)]

extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
extern crate atomicwrites;
extern crate csv;
extern crate eta3_spline;
extern crate libc;

use self::Msg::*;
use atomicwrites::{AllowOverwrite, AtomicFile};
use eta3_spline::eta_3;
use eta3_spline::{EtaParam, MotionState};
use gtk::Orientation::{Horizontal, Vertical};
use gtk::{GtkWindowExt, Inhibit, LabelExt, OrientableExt, RangeExt, WidgetExt};
use relm::{timeout, Relm, Widget};
use relm_attributes::widget;
use std::fs::File;
use std::io::prelude::*;
use std::process::*;

pub struct Eta3TestParams {
    pub x: f64,
    pub y: f64,
    pub t1: f64,
    pub t2: f64,
    pub k1: f64,
    pub k2: f64,
    pub dk1: f64,
    pub dk2: f64,
    pub eta1: f64,
    pub eta2: f64,
    pub eta3: f64,
    pub eta4: f64,
    pub eta5: f64,
    pub eta6: f64,
}

// Define the structure of the model.
pub struct Model {
    params: Eta3TestParams,
    file: AtomicFile,
}

// The messages that can be sent to the update function.
#[derive(Msg)]
pub enum Msg {
    Quit,
    Show,
}

#[widget]
impl Widget for Win {
    // The initial model.
    fn model(relm: &Relm<Self>, _: ()) -> Model {
        timeout(relm.stream(), 200, || Show);
        Model {
            params: Eta3TestParams {
                x: 1.,
                y: 1.,
                t1: 0.,
                t2: 0.,
                k1: 0.,
                k2: 0.,
                dk1: 0.,
                dk2: 0.,
                eta1: 2.,
                eta2: 2.,
                eta3: 0.,
                eta4: 0.,
                eta5: 0.,
                eta6: 0.,
            },
            file: AtomicFile::new("eta3.csv", AllowOverwrite),
        }
    }

    fn init_view(&mut self) {
        self.xslider.set_range(0.0, 10.);
        self.xslider.set_value(5.0);
        self.yslider.set_range(0.0, 10.);
        self.yslider.set_value(5.0);
        self.t1slider.set_range(0.0, 2. * std::f64::consts::PI);
        self.t2slider.set_range(0.0, 2. * std::f64::consts::PI);
        self.k1slider.set_range(-10., 10.0);
        self.k2slider.set_range(-10., 10.0);
        self.dk1slider.set_range(-10., 10.0);
        self.dk2slider.set_range(-10., 10.0);
        self.eta1slider.set_range(0.01, 10.0);
        self.eta1slider.set_value(5.0);
        self.eta2slider.set_range(0.01, 10.0);
        self.eta2slider.set_value(5.0);
        self.eta3slider.set_range(-10., 10.0);
        self.eta4slider.set_range(-10., 10.0);
        self.eta5slider.set_range(-100., 100.0);
        self.eta6slider.set_range(-100., 100.0);
    }

    // Update the model according to the message received.
    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
            Show => {
                self.model.params.x = self.xslider.get_value();
                self.model.params.y = self.yslider.get_value();
                self.model.params.y = self.yslider.get_value();
                self.model.params.t1 = self.t1slider.get_value();
                self.model.params.t2 = self.t2slider.get_value();
                self.model.params.k1 = self.k1slider.get_value();
                self.model.params.k2 = self.k2slider.get_value();
                self.model.params.dk1 = self.dk1slider.get_value();
                self.model.params.dk2 = self.dk2slider.get_value();
                self.model.params.eta1 = self.eta1slider.get_value();
                self.model.params.eta2 = self.eta2slider.get_value();
                self.model.params.eta3 = self.eta3slider.get_value();
                self.model.params.eta4 = self.eta4slider.get_value();
                self.model.params.eta5 = self.eta5slider.get_value();
                self.model.params.eta6 = self.eta6slider.get_value();
                self.model
                    .file
                    .write(|f| render(&self.model.params, f))
                    .ok();
            }
        }
    }

    view! {
        gtk::Window {
            property_default_width: 300,
            title: "Eta3 Visualizer",
            gtk::Box {
                orientation: Vertical,
                gtk::Label {
                    text: "x",
                },
                #[name="xslider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
                gtk::Label {
                    text: "y",
                },
                #[name="yslider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
                gtk::Label {
                    text: "t1",
                },
                #[name="t1slider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
                gtk::Label {
                    text: "t2",
                },
                #[name="t2slider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
                gtk::Label {
                    text: "k1",
                },
                #[name="k1slider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
                gtk::Label {
                    text: "k2",
                },
                #[name="k2slider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
                gtk::Label {
                    text: "dk1",
                },
                #[name="dk1slider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
                gtk::Label {
                    text: "dk2",
                },
                #[name="dk2slider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
                gtk::Label {
                    text: "eta1",
                },
                #[name="eta1slider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
                gtk::Label {
                    text: "eta2",
                },
                #[name="eta2slider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
                gtk::Label {
                    text: "eta3",
                },
                #[name="eta3slider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
                gtk::Label {
                    text: "eta4",
                },
                #[name="eta4slider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
                gtk::Label {
                    text: "eta5",
                },
                #[name="eta5slider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
                gtk::Label {
                    text: "eta6",
                },
                #[name="eta6slider"]
                gtk::Scale {
                    orientation: Horizontal,
                    value_changed => Show,
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    let mut dummy = File::create("eta3.csv").unwrap();
    write!(&mut dummy, "x,y\n0,0\n1,1").unwrap();
    let script = include_bytes!("../autoreplot.gp");
    let mut f = File::create("eta3.gp").unwrap();
    f.write(&*script).unwrap();
    let gnuplot = Command::new("gnuplot")
        .arg("autoreplot.gp")
        .spawn()
        .expect("Could not start gnuplot");
    Win::run(()).expect("Win::run failed");
    unsafe { libc::kill(gnuplot.id() as i32, libc::SIGINT) };
}

fn render(p: &Eta3TestParams, file: &mut File) -> Result<(), csv::Error> {
    let s = MotionState {
        x: 0.,
        y: 0.,
        t: p.t1,
        k: p.k1,
        dk: p.dk1,
    };
    let e = MotionState {
        x: p.x,
        y: p.y,
        t: p.t2,
        k: p.k2,
        dk: p.dk2,
    };

    let eta = EtaParam::new(p.eta1, p.eta2, p.eta3, p.eta4, p.eta5, p.eta6);
    let curve = eta_3(&s, &e, &eta);

    let mut wtr = csv::Writer::from_writer(file);
    wtr.write_record(&["x", "y"])?;
    let pts = curve.render(100);
    pts.iter().for_each(|p| wtr.serialize(p).unwrap());
    Ok(())
}
