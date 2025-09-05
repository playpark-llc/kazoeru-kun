use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::env;
use crate::models::status::Status;
use gloo_net::http::Request;
use serde_json::Value;

#[function_component]
pub fn PushAfter() -> Html {
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
    let push = {
        move |_| {
            spawn_local(async move {
                Request::post(&format!("{}/{}", env::APP_API_ROOT, "session"))
                    // .credentials(web_sys::RequestCredentials::Include)
                    .send()
                    .await;
            });
        }
    };

    html! {
        <div class="w-screen h-screen relative bg-primary-check">
            <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-[72.2%]">
                <div class="flex flex-col justify-center items-center space-y-9">
                    <div class="flex flex-col items-center space-y-4">
                        <img src="/images/sun.svg" />
                        <img src="/images/main-text.svg" class="w-full"/>
                    </div>
                    <a class="w-48 hover:-translate-y-1 cursor-pointer" onclick={ push }>
                        <img src="/images/status/dirty.png" />
                    </a>
                </div>
            </div>
            <div class="mb-12 absolute w-full flex justify-center bottom-0">
                <a href="detail">
                    <img src="/images/watch.svg" />
                </a>
            </div>
        </div>
    }
}