#Example of a back buffer in x11 with Rust.

I don't usually write stuff up as I don't think they'd be significantly interesting to anyone but me. However I found this kind of fun to do so I thought I'd try and put something down in words.

## Project: Game in Rust

So I recently discovered a programmer called Casey Muratori who I've been finding rather fun through various YouTube videos. Long story short this led me to find his series A Handmaid Hero which I kind of wanted to give a go. There's two problems with this though. One the entire series revolves around Windows and while I do have Windows I barely use it and find it rather obnoxious to code on as a platform. Two.) The series is in C. While I'm open to learning C properly at some point at the start of this year I decided I wanted to give a language a proper shot. Basically code in it for a year and then see if I enjoy it or not. That language was Rust and I don't want to be learning two languages at once.

This led me to a question. Could I take the essence of what Handmade Hero is and transform it to be in Rust and run on Linux? Welp, I'm less than a month into learning Rust so now seems as good a time as any to start, right? This won't be hard... nope, not at all.. it may also involve Cows. I have no idea why this involves Cows. Stop asking about the Cows.

## Displaying a Back Buffer to a Window.

Right, so the first few videos if I remember correctly, ignoring the intro to C ones involved getting a window to display an image to the screen. Obviously for Casey this involved using Windows which has a built in Window Manager.. Linux, being a Kernal does not. So I needed to pick one and while I know Wayland is a thing, I also know that x11 is also a thing and decided to choose x11. I figured this was significantly low level as to be in the spirit of the series.

I needed to find a way of exposing x11 to Rust which did involve a library to do some bindings. I found x11rb which is a basic Rust wrapper for the C Library as far as I'm aware so I'm using that.

The first part of this was relatively straight forward. Get a Window to display. There's a pretty nice example of this in the tutorial.rs file, so I just stuck in example6, renamed it to build_window() and moved on with my life. Okay, I didn't do that, originally I was trying to do it myself and in my first mistake attempted to be a clever boy and use a struct. This bit me in the arse. I did get the window to display this way but it caused me all kind of headaches as I progressed and eventually I just binned it, stuck everything in main.rs and used Example 6. On the plus side it did give me an understanding of what was happening with the window creation.

### Window Creation.

The way x11 works, I think, is it's basically a server/client modal. The server does the actual displaying of stuff to the screen and the client then sends requests in the form of packets to the server to be like - do this, do that, make stuff happen. So to begin with we need to open a connection the the server and at the end we can drop this connection.

Let's do this, so we open a connection to the server and we get back a tuple containing the connection we'll be using and a screen number. I assume the screen number is if we have multiple monitors connected or something but I don't so for me it's just a value I need to use at certain points. The connection is what we'll be using to send our requests with. At the end of the program we need to drop this.
```
let (conn, screen_num) = x11rb::connect(None)?;

drop(conn);
```

(Just as an aside if you are expecting this to be a detailed exploration of x11, Rust, and backbuffers, I'll have you know this is like based on a fortnight of me learning Rust, and 3 days of x11. This is an idiots perpective on the stuff the idiot did to get a thing to work. We are not thinking with portals here, we are thinking with wheels that have multiple edges.)

Next we have some basic window setup. Most of this is taken from the example at present barring me adding a few things like properties to set the window tile. There's more properties like icons but I didn't want to go too nuts at this stage in the process.

```
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

// map our window to the screen.
conn.map_window(win)?;

loop {
    //Handle stuff here like events and drawing.
}

drop(conn)?;
```

I've modified this from the code I've included in this repo to just give you the basic stuff of Window management. Some of the interesting bits include the CreateWindowAux. Here we can set what kind of events we want to get passed to the Window. At this point I'm just including EXPOSURE and KEY_PRESS. Exposure deals with when new parts of the window become visible. I'll later on use this to resize the image I'm drawing too when we resize the window.


