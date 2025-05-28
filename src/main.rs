//! Run with:
//!
//! ```sh
//! dx serve --platform web
//! ```

#![allow(non_snake_case, unused)]
use dioxus::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

type CommentType = String;
static CSS: Asset = asset!("assets/bulma/css/bulma.css");
fn error_page(error: impl Into<String> + std::fmt::Display) -> Element {
    rsx! {
        document::Stylesheet { href: CSS }
        p { class: "title is-1", "Error" }
        p { class: "subtitle is-3", "{error}" }
    }
}

#[component]
fn NoteModal(modal_open: Signal<Option<usize>>, notes: Signal<Vec<Note>>) -> Element {
    let index = modal_open().unwrap();

    let mut temp_text = use_signal(|| notes.read()[index].text.clone());

    // notes.read()[index].text

    let save_disabled = temp_text().is_empty();
    rsx! {
        div { class: "modal is-active",
            div { class: "modal-background" }
            div { class: "modal-card",
                header { class: "modal-card-head",
                    p { class: "modal-card-title", "Edit Note :3" }
                    button {
                        class: "delete is-large",
                        onclick: move |_| { modal_open.set(None) },
                    }
                }

                section {

                    textarea {
                        class: "textarea has-fixed-size subtitle",
                        value: temp_text(),
                        oninput: move |event| { temp_text.set(event.value()) },
                    }
                }
                footer { class: "modal-card-foot",
                    div { class: "buttons",
                        button {
                            class: "button is-success",
                            disabled: save_disabled,
                            onclick: move |_| async move {
                                notes.write()[index].text = temp_text();
                                modal_open.set(None);
                                note_update(index, notes.read()[index].clone()).await;
                            },
                            "Save"
                        }
                        button {
                            class: "button is-danger",
                            onclick: move |_| async move {
                                notes.write().remove(index);
                                note_delete(index).await;
                                modal_open.set(None);
                                note_delete(index).await;
                            },
                            "Delete"
                        }
                                        // button { class: "button is-success", "Cancel" }

                    }
                }
            }
        }
    }
}

#[component]
fn NoteItem(index: usize, modal_open: Signal<Option<usize>>, notes: Signal<Vec<Note>>) -> Element {
    let mut text = notes.read()[index].text.clone();
    if text.is_empty() {
        text = "[EMPTY]".to_string();
    }

    let disabled = notes.read()[index].disabled;
    let style = if disabled { "opacity: 0.4;" } else { "" };

    rsx! {
        div { class: "block", style: "{style}",
            div { class: "columns is-gapless px-2 is-mobile",
                div { class: "column is-1",
                    input {
                        r#type: "checkbox",
                        oninput: move |event| async move {
                            let is_disabled = event.value() == "true";
                            notes.write()[index].disabled = is_disabled;
                            note_update(index, notes.read()[index].clone()).await;
                        },
                        checked: disabled,
                    }
                    div { class: "buttons column" }
                }

                div { class: "column is-auto",
                    p {
                        class: "subtitle",
                        onclick: {
                            move |_| {
                                if !disabled {
                                    modal_open.set(Some(index));
                                }
                            }
                        },
                        "{text}"
                    }
                }

                if !disabled {


                    button {
                        class: "delete is-large column",
                        onclick: move |_| async move {
                            note_delete(index).await;
                            notes.write().remove(index);
                        },
                    }
                }
            }
        }
    }
}

fn app() -> Element {
    // let global_error = use_signal(||None);

    let mut modal_open = use_signal(|| None);
    let mut url = use_signal(String::new);
    // load futures on startup
    let server_future = use_server_future(notes_read)?;

    let Some(Ok(notes)) = server_future.read().clone() else {
        return error_page("Could not connect to Dioxus backend");
    };

    let mut notes = use_signal(|| notes);
    let vec_read = notes.read();
    let items_rendered = vec_read.iter().enumerate().map(|(index, a)| {
        rsx! {
            NoteItem { index, modal_open, notes }
        }
    });

    rsx! {
        document::Stylesheet { href: CSS }

        br {}

        if modal_open().is_some() {
            NoteModal { modal_open, notes }
        }

        {items_rendered}

        a {
            class: "button  is-large is-fullwidth",
            onclick: move |_| async move {
                notes.write().push(Default::default());
                note_add().await;
            },
            "+"
        }
    }
}
// #[cfg(feature = "server")]
#[cfg(feature = "server")]
const DB_PATH: &'static str = "db.json";
#[cfg(feature = "server")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Db {
    pub notes: Vec<Note>,
}
#[cfg(feature = "server")]
impl Db {
    pub fn save(&self) {
        std::fs::write(DB_PATH, serde_json::to_string_pretty(&self).unwrap()).unwrap();
    }
}

#[cfg(feature = "server")]
use lazy_static::lazy_static;
#[cfg(feature = "server")]
use std::sync::Arc;
#[cfg(feature = "server")]
use tokio::sync::RwLock;

#[cfg(feature = "server")]
lazy_static! {
    /// This is an example for using doc comment attributes
    static ref DB: Arc<RwLock<Db>> = Arc::new(RwLock::new(serde_json::from_slice(&std::fs::read(DB_PATH).unwrap()).unwrap()));
}



// fn(&T) -> bool
fn default<T: Default + std::cmp::PartialEq>(i: &T) -> bool {
    i == &Default::default()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Note {
    #[serde(default)]
    #[serde(skip_serializing_if = "default")]
    pub text: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "default")]
    pub disabled: bool,
}

// CRUD
#[server(endpoint = "note_add")]
async fn note_add() -> Result<(), ServerFnError> {
    DB.write().await.notes.push(Default::default());
    DB.read().await.save();
    Ok(())
}

#[server(endpoint = "server_valid")]
async fn server_valid() -> Result<String, ServerFnError> {
    Ok("hello! :D".to_string())
}

#[server(endpoint = "note_update")]
async fn note_update(index: usize, note: Note) -> Result<(), ServerFnError> {
    // check if note exists
    if DB.read().await.notes.get(index).is_none() {
        return Err(ServerFnError::Args(String::from(
            "Invalid index at array! note does not exist",
        )));
    };
    DB.write().await.notes[index] = note;
    DB.read().await.save();
    Ok(())
}


#[server(endpoint = "notes_read")]
async fn notes_read() -> Result<Vec<Note>, ServerFnError> {
    println!("ping!");
    Ok(DB.read().await.notes.clone())
}

#[server(endpoint = "note_delete")]
async fn note_delete(index: usize) -> Result<(), ServerFnError> {
    DB.write().await.notes.remove(index);
    DB.read().await.save();
    Ok(())
}



fn main() {
    #[cfg(not(feature = "server"))]
    server_fn::client::set_server_url("https://test.toastxc.xyz");

    dioxus::launch(app);
}
