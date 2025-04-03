use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Main {}
    }
}

#[component]
fn Main() -> Element { 

    let toggle_post_form = use_signal(|| false);
    let post_title = use_signal(String::new);
    let post_body = use_signal(String::new);
    let image_data = use_signal(|| Vec::<u8>::new());
    let mut posts = use_signal(Vec::<Post>::new);
    let refresh_signal = use_signal(|| 0); 

    let _ = use_resource(move || async move {

        refresh_signal(); 

        posts.clear();

        for post in get_posts().await {
            posts.write().push(post);
        }
    });

    rsx! {
        MenuComponent { toggle_post_form },

        if toggle_post_form() {
            PostEntryComponent { post_title, post_body, image_data, refresh_signal, toggle_post_form }
        }else{
            PostsComponent { posts }
        } 
    }
}

#[component]
fn MenuComponent(toggle_post_form : Signal<bool>) -> Element {
    rsx! {
        div {
            class : "menu-container",
            button { 
                onclick : move |_| {
                    if toggle_post_form() {
                        toggle_post_form.set(false);
                    }else{
                        toggle_post_form.set(true);
                    }
                },

                if toggle_post_form(){
                    "< Back"
                }else{
                    "+ Create Post"
                }
            },
            ul {
                li { "Home" },
                li { "Popular" }
                li { "Categories" }
                li { "About me" }
            }
        }
    }
}

#[component]
fn PostsComponent(posts : Signal<Vec<Post>>) -> Element {
    rsx! {
        main {
            div {
                class : "main-inner-container",

                for post in posts.iter() {
                    PostComponent { post : post.clone() }
                }
            }
        },
    }
}

#[component]
fn PostComponent(post : Post) -> Element {

    rsx!(
        div {
            class : "post-container",
            div {
                class : "post-header",
                h1 { {post.title} }
            },

            if let Some(image_file) = post.image_file {
                div {
                    img {
                        src : format!("http://127.0.0.1:8081/images/{}", image_file),
                    }
                }
            }

            div { 
                class : "post-content",
                div { 
                    class: "post-body",
                    PostBody { body : post.post_body }
                } 
            },
            div {
                class : "post-footer",
                span {
                    "Alexandre"
                },
                label { 
                    { post.created_time }
                }
            }
        }
    )
}

#[component]
fn PostBody(body : String) -> Element {
    rsx!{
        for line in body.split("\n"){

            if line.get(0..3).or_else(|| Some("") ).unwrap() == "###"{
                h2 {
                    { line.get(3..).unwrap() }
                }
            }
            else if line.get(0..2).or_else(|| Some("") ).unwrap() == "##"{
                h3 {
                    { line.get(2..).unwrap() }
                }
            }else{
                p {
                    {line}
                }
            }
        }    
    } 
}

#[component]
fn PostEntryComponent(
    post_title : Signal<String>, 
    post_body : Signal<String>, 
    image_data : Signal<Vec<u8>>, 
    toggle_post_form : Signal<bool>,
    refresh_signal: Signal<i32>) -> Element {

    rsx!(
        main {
            div {
                class : "main-inner-container",
                div {
                    class : "input-container",
                    input {
                        type : "text",
                        placeholder : "Post title",
                        oninput : move |event| {
                            post_title.set(event.value());
                        }
                    }
                }
                div {
                    class : "input-container",
                    textarea { 
                        class : "textarea",
                        placeholder : "Post body",
                        oninput : move |event| {
                            post_body.set(event.value());
                        }
                    }
                }
                div {
                    class : "actions",

                    label {
                        for : "file-upload",
                        class : "upload-button",
                        "Upload Image"
                    },

                    input {
                        id : "file-upload",
                        type: "file",
                        accept: "image/*",
                        onchange: move |event| async move {
                            let engine = event.files().unwrap();

                            if let Some(file) = engine.files().get(0){
                                let data = engine.read_file(file).await.unwrap();

                                image_data.set(data);
                            }
                        },
                        "Upload Image"
                    },

                    button {
                        onclick : move |_| async move{
                            if image_data().len() > 0 {
                                create_post(post_title(), post_body(), Some(image_data())).await;
                            }else{
                                create_post(post_title(), post_body(), None).await;
                            }

                            refresh_signal.set(refresh_signal() + 1);
                            toggle_post_form.set(false);
                        },
                        "Create Post"
                    }
                }
            }
        }
    )
}

async fn create_post(post_title : String, post_body : String, image_data : Option<Vec<u8>> ) {

    let post = PostEntry {
        title : post_title,
        post_body : post_body,
        image_data : image_data
    };

    reqwest::Client::new()
        .post("http://127.0.0.1:8081/create_post")
        .json(&post)
        .send()
        .await
        .unwrap();
}

async fn get_posts() -> Vec<Post> {
    reqwest::get("http://127.0.0.1:8081/get_posts")
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Post {
    id : u32,
    title : String,
    post_body : String,
    image_file : Option<String>,
    created_time : String
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PostEntry {
    title : String,
    post_body : String,
    image_data : Option<Vec<u8>>
}