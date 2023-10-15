use std::collections::VecDeque;

use tracing::error;
use wasm_bindgen::{
    closure::Closure,
    convert::{FromWasmAbi, IntoWasmAbi},
    JsCast,
};
use web_sys::{MessageEvent, WebSocket};
use yew::prelude::*;

use crate::{
    gsi::{GameState, Team},
    log,
};
use csgsi_shared::Message;

fn closure<T: FromWasmAbi + 'static, U: IntoWasmAbi + 'static>(
    f: impl Fn(T) -> U + 'static,
) -> Closure<dyn Fn(T) -> U> {
    Closure::new(Box::new(f))
}

#[derive(Clone, Default, PartialEq)]
struct State {
    game_state: GameState,
    killfeed: VecDeque<String>,
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
    let state = use_state(State::default);

    let incoming_msg = use_state(|| None::<String>);

    {
        let socket = socket.clone();
        let state = state.clone();
        let incoming_msg = incoming_msg.clone();
        let incoming_msg2 = incoming_msg.clone();

        // TODO: this is getting messy. Consider a reducer.
        use_effect_with((incoming_msg, state), move |(incoming_msg, state)| {
            let Some(payload) = &**incoming_msg else {
                return;
            };
            let msg = match serde_json::from_str(payload) {
                Ok(m) => m,
                Err(e) => {
                    error!("error deserializing message: {:?}", e);
                    return;
                }
            };
            match msg {
                Message::State(raw_state) => match serde_json::from_str(raw_state.get()) {
                    Ok(new_game_state) => {
                        state.set(State {
                            game_state: new_game_state,
                            killfeed: state.killfeed.clone(),
                        });
                    }
                    Err(e) => error!("error deserializing state: {:?}", e),
                },
                Message::Log(l) => match log::Event::try_from(&*l) {
                    Ok(log::Event::Kill {
                        killer,
                        target,
                        weapon,
                    }) => {
                        let mut kf = state.killfeed.clone();
                        kf.push_back(format!(
                            "({}) {} [{}] ({}) {}",
                            killer.team, killer.name, weapon, target.team, target.name
                        ));
                        state.set(State {
                            killfeed: kf,
                            ..(**state).clone()
                        });
                    }
                    Err(e) => error!("error parsing log event: {:?}", e),
                },
            }
            incoming_msg.set(None);
        });

        use_effect_with((), move |_| {
            let on_message = closure(move |e: MessageEvent| {
                let s = e
                    .data()
                    .as_string()
                    .to_owned()
                    .expect("expected websocket message to be text");
                incoming_msg2.set(Some(s));
            })
            .into_js_value();
            // TODO: connection management (retry, reconnect)
            let ws = WebSocket::new("ws://localhost:3000/ws").unwrap();
            ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            socket.set(Some(ws));
        });
    };

    let game_state = &state.game_state;

    html! {
        <div>
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
            <div id="killfeed">
                {state.killfeed.iter().map(|msg| html!{
                    <div>{msg}</div>
                }).collect::<Html>()}
            </div>
        </div>
    }
}
