# Example of a back buffer in x11 with Rust.

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

I've modified this from the code I've included in this repo to just give you the basic stuff of Window management. Some of the interesting bits include the CreateWindowAux. Here we can set what kind of events we want to get passed to the Window. At this point I'm just including EXPOSURE and KEY_PRESS. Exposure deals with when new parts of the window become visible. I'll later on use this to resize the image I'm drawing when we resize the window. (I plan to delve into a bit more detail about this later but want to talk about some of the more interesting stuff in the project first.)

## Setting up a back buffer.

Okay, so this is were I started to run into a few issues. As I mentioned a Rust programmer I am not and also this is my first go at this library in any detail. Before last weekend my understanding of x11 was 'that's the window manager thing, right?' so we are talking a fairly limited knowledge base to go on here. As further hindrance I was relying on a... tutorial, it's this:

https://github.com/psychon/x11rb/blob/master/x11rb/examples/tutorial.rs

You may notice that it doesn't really go into pixel manipulation in much detail, or back buffers, or anything that I actually wanted to do apart from the basic window management stuff. So, this led me on a little journey. I'd like to claim it was a lovely romp through some green fields on a summer's day but it was more like trudging through a swamp in the middle of January. Basically I had to go digging through the x11rb git repository to find anything vaguely like what I was after. It took a while and there were a few false starts. Full disclosure, I did at one point point gemini at it a be like "How do I import shm?" after finding something in C that looked about right, this was a false start though. It also led me to discover AI is very good at giving you the correct function but the wrong signatures.

Eventually I stumbled across Image which is what I'm using for this. It's pretty neat and doesn't seem to be mentioned in that rs file so I wanted to show it. This is my image generation code:

```
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
```
So this is really fun and super neat, I think. I've made a function call that takes a width, a height, and a WindowProperties struct I created. This contains the background_color I want for the image as well as the width and height because I'm a dumb ass and didn't modify the signature. So *ahem* yes, erm that's good.

Okay, so first we want our pixel data. I only really care about the RGB values here, the fourth value is just padding although it might be useful for an alpha channel, maybe, but apparently it's better to have 32 bits, than 2.. I want to say 4, 24. Yes, 32 - 8 is indeed 24. I am the smarts.

So to start with we need a vector of bytes, for every pixel we need four bytes. the red, the green, the blue, and finally the padding. To get this we times the width of our window by the height of the window by the amount of bytes per peixel. We then want to initialise this to our background colour. `for chunk in data.chunks_exact_mut(4)` all this is doing is looping through our array but instead of giving us a value back one at a time it gives us an array of the 4 bytes. Useful as it would be annoying if I had to count it myself. I then just set these to correspond to our background color. Note: I'm having to do blue, green, red here. I think I could fix that but it's not that big a deal.

Next we have scanline_pad, and bits_per_pixel, there's also 24 which I believe is the depth. I didn't name that for some reason I'll probably fix that and get rid of the magic number. By the way. This is a royal pain in the butt. These numbers / enums need to be right. If they are not right x11 does this really great thing of just not drawing the image. It doesn't like error or anything, you just don't get the image. Which is lovely for debugging.

Then Cow. A Cow happens. I did warn you about the Cows. You were foretold there would be a Cows. So what is a Cow I hear you ask. According to the rust book a Cow is a smart pointer offering clone on write functionality. I have no idea why you'd call... ah, clone on write. Cow. See, we are all learning things today. So anyway, we take our vec and make it into a Cow, as you do and then create our image and return it back. This is all fun and games I hear you cry but what's so good about an image?

Let's look at my draw function:

```
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
```

Tada! We can draw to individual pixels in the image. We effectively have a back buffer. Like you have no idea how much joy I had when I got this to work. Is it shit? Probably. Is there a better way to do this? Certainly. Do I think it's fucking awesome? Yes. There's things I think I could do to improve this in some ways. One, I think I need to kill the Cow. Like clone-on-write sounds terrible and I'm certain there's a way to just use a borrowed reference to do this. Secondly, in the image.rs file of the github library it mentions Image::Native. Under the hood this image is being transformed into a format x11 likes and since I'm pushing it every frame I'd probably be better off transforming it when I create the image so that doesn't need to happen. Saying that rather pleased with myself for getting this to work. I've had very little to go off on this one and it's been super fun getting to mess around in two things I barely know. It's been at the level of enjoyment of when you first get a sprite to move across a screen.

Like programming can be hard, frustrating, and annoying all boiled into one but like the absolutely thrill of getting something to work.

Anyway, I just wanted to put this code here as like I said I think it's kind of cool and I enjoyed writing it. I'm probably going to try and work out some Audio stuff next. I'm not sure how that "works" in Linux. Like I need a thing on this level that does audio were I can just manipulate rawish data I'm not certain what that is or looks like in practice.

For this some things I'd like to do in future.

1.) Kill the Cow.
2.) In the Handmade Hero series he uses something called StretchDiBits if I recall correctly which does Image stretching. I think that'd maybe be interesting to try and implement.
3.) Probably clean this up a bit. There's a lot of magic happening which I dislike.
4.) Actually learn Rust so I can find out all the ways in which this is wrong and terrible and bad.

It's been fun. Laters.

