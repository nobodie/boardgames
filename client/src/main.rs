#![feature(async_closure)]
#![feature(let_chains)]

use gloo_console::log;
use reqwest::header;
use serde_json::to_string;
use web_sys::{HtmlElement, HtmlInputElement, MessageEvent};

use yew::{platform::spawn_local, prelude::*};
#[function_component]
fn App() -> Html {
    let counter = use_state(|| "0".to_string());
    let increment = use_state(|| 1);

    {
        let counter = counter.clone();

        use_effect_with_deps(
            move |_| {
                log!("toto");
                spawn_local(async move {
                    if let Ok(res) = reqwest::get("http://localhost:3000/counter").await {
                        if let Ok(text) = res.text().await {
                            counter.set(text.to_string());
                        }
                    }
                });
                || {}
            },
            (),
        );
    }

    let onclick = {
        let counter = counter.clone();
        let increment = increment.clone();
        move |_| {
            let counter = counter.clone();
            let increment = increment.clone();

            spawn_local(async move {
                if let Ok(res) = reqwest::get(format!(
                    "http://localhost:3000/counter/increase/{}",
                    *increment
                ))
                .await
                {
                    if let Ok(text) = res.text().await {
                        counter.set(text.to_string());
                    }
                }
            })
        }
    };

    let onchange = {
        let increment = increment.clone();
        log!("onchange");
        Callback::from(move |e: InputEvent| {
            log!("onchange");
            if let Some(d) = e.data() {
                increment.set(d.parse::<i32>().unwrap_or(1));
            }
        })
    };

    html! {
        <div>
            <Button/>
            <button {onclick}>{ "+" }</button>
            <input type="text" oninput = {onchange} value={(*increment).to_string()}/>
            <p>{ (*counter).clone() }</p>
        </div>
    }
}

#[function_component]
fn Button() -> Html {
    html! {<p>{"toto"}</p>}
}

const GAMES: &[(&str, &str)] = &[("rps", "Rock-Paper-Scissor"), ("chess", "Chess")];

#[function_component]
fn Lobby() -> Html {
    html! {
        <div>
            <CreateNewRoomForm/>
        </div>
    }
}

#[function_component]
fn CreateNewRoomForm() -> Html {
    let game_id = use_state(|| GAMES[0].0.to_string());

    let onclick = {
        move |_| {
            todo!();
        }
    };

    /*let ongamechanged = {
        move |_| {
            todo!();
        }
    };*/

    let ongamechanged = {
        let game_id = game_id.clone();
        Callback::from(move |e: yew::html::onchange::Event| {
            log!(e.clone());

            let input = e.target_unchecked_into::<HtmlElement>();
            log!("titi", input.clone(), input.node_value());

            /*if input.is_some() {
                game_id.set(input.unwrap().value());
                log!("new value for game_id : ", (*game_id).clone());
            }*/
        })
    };

    html! {
        <form>

            <select list="games" disabled=false required=true /*onselect = {ongamechanged.clone()}*/  onchange = {ongamechanged}>
            {
                GAMES.into_iter().map(|(id, name)| {
                    if id.to_string() == *game_id {
                        html!{<option value={id.to_string()} selected=true>{name.to_string()}</option>}
                    } else {
                        html!{<option value={id.to_string()} >{name.to_string()}</option>}
                    }
                }).collect::<Html>()
            }
            </select>

            <button {onclick}>{ "Create new game room" }</button>

        </form>
    }
}

fn main() {
    yew::Renderer::<Lobby>::new().render();
}
