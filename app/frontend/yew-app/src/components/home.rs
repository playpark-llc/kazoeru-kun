use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::router::Route;
use crate::env;
use crate::models::status::Status;
use gloo_net::http::Request;
use serde_json::Value;

#[function_component]
pub fn Home() -> Html {
    let status_state = use_state(|| Status::Clean); // 初期値は適宜
    {
        let status_state = status_state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&format!("{}/{}", env::APP_API_ROOT, "status_history"))
                    .send()
                    .await
                {
                    // 1) JSONとしてパースして、場所に応じて取り出す
                    if let Ok(json) = resp.json::<Value>().await {
                        let s = json.get("result")
                            .and_then(|r| r.get("current_status"))
                            .and_then(|cs| cs.get("status"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("CLEAN");

                        let mut status = Status::from(s);

                        status_state.set(status);
                    }
                }
            });
            || ()
        });
    }

    let navigator = use_navigator().unwrap();
    let push = {
        let navigator = navigator.clone();
        move |_| {
            let navigator = navigator.clone();
            spawn_local(async move {
                Request::post(&format!("{}/{}", env::APP_API_ROOT, "session"))
                    // .credentials(web_sys::RequestCredentials::Include)
                    .send()
                    .await;
                {
                    navigator.push(&Route::PushAfter);
                }
            });
        }
    };

    html! {
        <div class="w-screen h-screen bg-primary-check">
            <div style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -72.2%);">
                <div class="flex flex-col justify-center items-center space-y-9">
                    <img src="/images/main-text.svg" />
                    <a class="w-48 hover:-translate-y-1 cursor-pointer" onclick={ push }>
                        <img src={ status_state.image_url() } />
                    </a>
                </div>
            </div>
        </div>
    }
}
