use tracing::error;
use wasm_bindgen::{
    closure::Closure,
    convert::{FromWasmAbi, IntoWasmAbi},
    JsCast,
};
use web_sys::{MessageEvent, WebSocket};
use yew::prelude::*;

use crate::gsi::{GameState, Team};

fn closure<T: FromWasmAbi + 'static, U: IntoWasmAbi + 'static>(
    f: impl Fn(T) -> U + 'static,
) -> Closure<dyn Fn(T) -> U> {
    Closure::new(Box::new(f))
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <main>
            <Hud />
        </main>
    }
}

#[function_component(Hud)]
pub fn hud() -> Html {
    let socket = use_state(|| None);
    let game_state = use_state(|| None::<GameState>);

    {
        let socket = socket.clone();
        let game_state = game_state.clone();
        use_effect_with((), move |_| {
            let on_message = closure(move |e: MessageEvent| {
                let s = e
                    .data()
                    .as_string()
                    .expect("expected websocket message to be text");
                match serde_json::from_str(&s) {
                    Ok(new_game_state) => game_state.set(new_game_state),
                    Err(e) => error!("error deserializing message: {:?}", e),
                }
            })
            .into_js_value();
            // TODO: connection management (retry, reconnect)
            let ws = WebSocket::new("ws://localhost:3000/ws").unwrap();
            ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            socket.set(Some(ws));
        });
    };

    let Some(game_state) = &*game_state else {
        return html! {};
    };

    html! {
        <div id="players">
            {[Team::T, Team::CT].into_iter().map(|team| {
                let players = game_state.players(team);
                html!{
                    <div class="team">
                        <header>{team.to_string()}</header>
                        {players.iter().map(|p| {
                            html!{
                                <div class={classes!("player", (p.state.health == 0).then_some("dead"))}>
                                    {&p.name} {" ❤️ "} {&p.state.health}
                                    <ul class="match-stats">
                                        <li>
                                            <span class="name">{"🔫"}  </span> <span class="value">{&p.match_stats.kills}</span>
                                        </li>
                                        <li>
                                            <span class="name">{"💀"} </span> <span class="value">{&p.match_stats.deaths}</span>
                                        </li>
                                        <li>
                                            <span class="name">{"🤝"}</span> <span class="value">{&p.match_stats.assists}</span>
                                        </li>
                                    </ul>
                                </div>
                            }
                        }).collect::<Html>()}
                    </div>
                }
            }).collect::<Html>()}
        </div>
    }
}
