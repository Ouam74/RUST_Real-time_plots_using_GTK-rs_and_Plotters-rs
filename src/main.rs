extern crate gtk;
extern crate glib;
extern crate gio;
extern crate cairo;
use plotters::prelude::*;
use std::env::args;
use gtk::prelude::*;
use gio::prelude::*;
use rand::Rng;
use std::{thread, time};
use std::time::Duration;
use std::sync::{Arc, Mutex};


fn build_ui(application: &gtk::Application){
    let glade_src = include_str!("Plotter.glade");
    let builder   = gtk::Builder::from_string(glade_src);

    let window: gtk::Window = builder.get_object("Window1").expect("Couldn't get window");
    window.set_application(Some(application));

    let startbutton: gtk::Button = builder.get_object("Startbutton").expect("Couldn't get StartButton");
    let stopbutton: gtk::Button = builder.get_object("Stopbutton").expect("Couldn't get StopButton");
    let drawingarea: gtk::DrawingArea = builder.get_object("Drawingarea").expect("Couldn't get Drawingarea");

    // Create Plot ARGB32
    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 1000, 600).expect("Can't create surface"); //build a new ImageSurface to draw on
    let cr = cairo::Context::new(&surface); //build a new cairo context from that ImageSurface to draw on

    // Signal / Slots
    drawingarea.connect_draw(move|_, context| draw_fn(&context)); // plots drawn_fn into drawing_area

    let abort = Arc::new(Mutex::new(0)); // allow `abort` to be shared across threads (Arc) and modified (this is a pointer)

    let drawingarea_clone = drawingarea.clone();
    let cr_clone = cr.clone();
    let abort_clone = abort.clone(); // create a cloned reference before moving `abort` into the thread.
    startbutton.connect_clicked(move|_| startclicked(&drawingarea_clone, &cr_clone, &abort_clone)); // connects startbutton click with function startclicked

    let abort_clone = abort.clone();
    stopbutton.connect_clicked(move|_| stopclicked(&abort_clone)); // connects startbutton click with function startclicked

    window.show_all();
}

fn startclicked(drawingarea: &gtk::DrawingArea, cr: &cairo::Context, abort: &Arc<Mutex<i32>>){
    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    *abort.lock().unwrap() = 0; // re-initialize "abort" to zero

    // sender
    thread::spawn(move || {
        println!("THREAD STARTED!!!!!!");
        loop { // for _i in 0..1001 {
            thread::sleep(Duration::from_micros(10)); // time to get the datas
            let val = 0.0; // println!("{:?}", _i);
            tx.send("val").unwrap(); // send (emit) 'i' to channel, receiver will be run on the main thread
        }
    });

   // Attach receiver to the main context and set the val from here
   let drawingarea_clone = drawingarea.clone();
   let abort_clone = abort.clone();
   rx.attach(None, move |val| { // val is the received value that was sent by the tx
       let abort = abort_clone.lock().unwrap();
       if *abort == 0 {
           drawingarea_clone.queue_draw(); // refresh drawingarea
           glib::Continue(true)
       }
       else {
           println!("THREAD TERMINATED!!!!!!");
           glib::Continue(false)
       }
   });
}

fn stopclicked(abort: &Arc<Mutex<i32>>){
    *abort.lock().unwrap() = 1; // this works --> let mut abort = abort.lock().unwrap(); *abort = 1; // set "abort" value to 1
}

fn createserie_iter() -> Vec<(f64, f64)> {
    let mut ret = vec![];
    let mut _rng = rand::thread_rng();
    for j in 0..1000 {
        ret.push( ( j as f64, _rng.gen::<f64>() ) );
    }
    ret
}

fn draw_fn(c: &cairo::Context) -> gtk::Inhibit { // plot data using plotters

    let mut rng = rand::thread_rng();
    let root = plotters_cairo::CairoBackend::new(&c, (600, 600)).unwrap().into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .margin(50)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0.0f64..1000.0f64, 0.0f64..1.0f64)
        .unwrap();

    chart.configure_mesh()
        .draw()
        .unwrap();

    let mut newserie = {
        let mut current: Vec<(f64, f64)>;
        current = createserie_iter();
        current
    };

    chart.draw_series(LineSeries::new(newserie, &RED));

    root.present();

    Inhibit(false)
}

// fn draw_fn(cr: &cairo::Context) -> gtk::Inhibit { // plot data using cairo
//     // paint canvas white
//     cr.set_source_rgb(0.0, 0.0, 0.0);
//     cr.paint();
//
//     // draw 1000 random points
//     cr.set_source_rgb(1.0, 1.0, 0.0);
//     cr.set_line_width(1.0);
//     for _i in 0..1000 {
//        let x = (_i as f64) * 1.0;
//        let y = rand::random::<f64>() * 600.0; // generate 1000 pts each time the draw_fn function is called
//        cr.line_to(x, y);
//     }
//     cr.stroke();
//     Inhibit(false)
// }


fn main() {

    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
    return;
    }

    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.grid"),
        Default::default(),
    )
    .expect("Initialization failed...");

    application.connect_activate(|app|{
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
