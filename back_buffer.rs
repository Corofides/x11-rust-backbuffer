use std::error::Error;

use x11rb::connection::Connection;
use x11rb::errors::ParseError;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::image::{Image,BitsPerPixel,ScanlinePad, ImageOrder};
use std::borrow::Cow;

struct WindowProperties {
    background_color: (u8, u8, u8),
    width: u16,
    height: u16,
    active: bool,
}

impl Default for WindowProperties {
    fn default() -> Self {
        WindowProperties {
            background_color: (255, 0, 0),
            width: 0,
            height: 0,
            active: true,
        }
    }
}

fn draw<'a>(mut image: Image<'a>, win_properties: &WindowProperties, offset: &u16) -> Image<'a> {
    for x in 0..win_properties.width {
        for y in 0..win_properties.height {
            let pix_color = u32::from_le_bytes([
                (x + offset % 256) as u8, //win_properties.background_color.2,
                (y % 256) as u8, //win_properties.background_color.1,
                0,  //win_properties.background_color.0,
                255,
            ]);
            image.put_pixel(x as u16, y as u16, pix_color);
        }
    }

    image
}


fn create_image_for_display(width: u16, height: u16, properties: &WindowProperties) -> Result<Image<'static>, ParseError> {

    let size: usize = width as usize * height as usize * 4 ;

    // Create a vector containing 4 bytes per pixel;
    let mut data = vec![0u8; size];

    for chunk in data.chunks_exact_mut(4) {
        chunk[0] = properties.background_color.2; //255; // Blue
        chunk[1] = properties.background_color.1; // Green
        chunk[2] = properties.background_color.0;
        chunk[3] = 255; //255; // padding / alpha
    }

    let scanline_pad = ScanlinePad::Pad32;
    
    let bits_per_pixel: BitsPerPixel = BitsPerPixel::B32;

    let cow = Cow::from(data);

    let image = Image::new(
        width,
        height,
        scanline_pad,
        24,
        bits_per_pixel,
        ImageOrder::LsbFirst,
        cow,
    );

    image
}

fn build_window() -> Result<(), Box<dyn Error>> {

    // Initialise a struct to contain some properties about the window
    // that we can reference.
    let mut win_properties = WindowProperties::default();

    let (conn, screen_num) = x11rb::connect(None)?;

    let setup = &conn.setup();
    let screen = &setup.roots[screen_num];

    let _win = screen.root;
    
    let win = conn.generate_id()?;

    let values = CreateWindowAux::default()
        .background_pixel(screen.white_pixel)
        .event_mask(
            EventMask::EXPOSURE | EventMask::KEY_PRESS
        );

    conn.create_window(
        24,
        win,
        screen.root,
        0,
        0,
        150,
        150,
        10,
        WindowClass::INPUT_OUTPUT,
        screen.root_visual,
        &values,
    )?;

    // create a graphics context for drawing our image to the screen.
    let foreground = conn.generate_id()?;
    let values = CreateGCAux::default()
        .foreground(screen.black_pixel)
        .graphics_exposures(0);

    conn.create_gc(foreground, win, &values)?;


    // change the title of our window.
    let title = "Example of a back buffer in x11";
    conn.change_property8(
        PropMode::REPLACE,
        win,
        AtomEnum::WM_NAME,
        AtomEnum::STRING,
        title.as_bytes()
    )?;
  
    let height: u16 = 100;
    let width: u16 = 100;

    // create an image so we have one, this should get overwritten immediately.
    let mut image = create_image_for_display(width, height, &win_properties)?;
    let mut offset = 0;

    // map our window to the screen.
    conn.map_window(win)?;

    while (win_properties.active) {
        let event = conn.poll_for_event()?;

        if let Some(event) = event {
            match event {
                Event::Expose(_) => {
                    // if we expose the window than create a new image of the height and width of
                    // the window to use which will handle resizing.
                    let geom = conn.get_geometry(win)?.reply()?;

                    let current_width: u16 = geom.width;
                    let current_height: u16 = geom.height;

                    if (win_properties.width != current_width) || (win_properties.height != current_height) {
                        win_properties.width = current_width;
                        win_properties.height = current_height;

                        image = create_image_for_display(win_properties.width, win_properties.height, &win_properties)?;
                    }
                },
                Event::KeyPress(event) => {
                    // Listen for the Escape key and if pressed exit the loop;
                    println!("Key pressed in window: {:?}", event.detail);
                    if event.detail == 9 {
                        win_properties.active = false;
                    }
                }
                _ => {/* Do nothing as we don't handle these events yet. */ }
            }
        }

        // We're using this as a cheap way of getting an offset for animation.
        offset = offset + 1;
        
        if win_properties.width > 0 {
            offset = offset % win_properties.width;
        }

        let properties = &win_properties;

        // Render loop.
        image = draw(image, properties, &offset);

        // This should maybe go in the render loop but unsure.
        image.put(&conn, win, foreground, 0, 0)?;
        conn.flush()?;

    }

    drop(conn);
    Ok(())
}

fn main() {
    let _ = build_window();
}
