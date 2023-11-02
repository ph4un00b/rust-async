use std::io::{self, Write};

#[macro_use]
extern crate rouille;

fn main() {
    println!("Now listening on localhost:8000");

    rouille::start_server("localhost:8000", move |req| {
        rouille::log(req, io::stdout(), || {
            router!(req,
                (GET) (/) => {
                    rouille::Response::html(PAGE)
                },
                (POST) (/upload) => {
                    upload_ctrl(req)
                },
                (GET) (/img/{name: String}) => {
                    println!("looking for: {name}");
                    if let Some(request) = req.remove_prefix("/img") {
                        return rouille::match_assets(&request, "out");
                    }
                    rouille::Response::html("404 error. Try again 😏.")
                        .with_status_code(404)
                },
                _ => rouille::Response::empty_404()
            )
        })
    });
}

fn upload_ctrl(req: &rouille::Request) -> rouille::Response {
    const MAX_FILE_SIZE: usize = 1_024 * 500 /*500 kb*/;
    // println!(">> {:?}", req.headers());
    // println!(">> {:?}", req.header("Content-length"));
    if let Some(string_size) = req.header("Content-length") {
        match string_size.parse::<usize>() {
            Ok(size) => {
                if size > MAX_FILE_SIZE {
                    println!(">> {size} bytes!");
                    return rouille::Response::empty_400();
                }
            }
            Err(_) => return rouille::Response::empty_400(),
        }
    }
    let data = try_or_400!(post_input!(req, {
        files: Vec<rouille::input::post::BufferedFile>,
    }));

    println!("Received data: {:?}", data);

    let mut imgs = vec![];

    for (num, hack) in data.files.into_iter().enumerate() {
        if hack.data.is_empty() {
            continue;
        }
        //? println!(">> {}",std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_millis());
        let id = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let filepath = format!("public/{num}-{id}.png");

        let mut file = std::fs::File::create(&filepath).expect("file creation");
        file.write_all(&hack.data).expect("write file");
        let resized = image::open(&filepath).unwrap();

        let id = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let name = format!("{num}-{id}.png");
        let filepath = format!("out/{name}");
        imgs.push(name.clone());
        resized
            .resize_exact(100, 100, image::imageops::FilterType::Nearest)
            //   .save_with_format(filepath, image::ImageFormat::WebP)
            .save(filepath)
            .unwrap();
    }

    let filename = imgs.pop().unwrap();
    rouille::Response::html(format!(
        "Success 🎉! try: <a href=\"img/{filename}\">{filename}</a>."
    ))
}

static PAGE: &str = r#"
<html lang="en">
<head>
<meta charset="UTF-8" />
<title>multipart demo</title>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css">
</head>
<body>
<main class="container">
        <h1>multipart demo</h1>
        <form id="form" action="upload" method="POST" enctype="multipart/form-data">
            <label>Select an img: </label>
            <input type="file" name="files" id="file_one" />
            <input type="file" name="files" id="file_two" />
            <input type="file" name="files" id="file_three" />
            <br />
            <p><button>Upload</button></p>
        </form>
      </main>
    </body>
</html>
"#;
