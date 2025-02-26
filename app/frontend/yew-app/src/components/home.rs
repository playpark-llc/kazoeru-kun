use yew::prelude::*;
use yew_router::prelude::*;
use crate::router::Route;
use wasm_bindgen_futures::spawn_local;
use crate::env;

#[function_component]
pub fn Home() -> Html {
    let response_data = use_state(|| "response".to_string());
    let navigator = use_navigator().unwrap();
    let push = {
        let response_data = response_data.clone();
        move |_| {
            let response_data = response_data.clone();
            spawn_local(async move {
                let client = reqwest::Client::new();
                let req = client
                    .post(format!("{}/{}", env::APP_API_ROOT, "session"))
                    .send()
                    .await
                    .expect("failed to send request");

                let data = req.text().await.expect("failed to get response");

                response_data.set(format!("{:?}", data));
            });
            navigator.push(&Route::PushAfter);
        }
    };

    html! {
        <div class="w-screen h-screen bg-primary-check">
            <div style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -72.2%);">
                <div class="flex flex-col justify-center items-center space-y-9">
                    <img src="/images/push-me.svg" />
                    <a class="w-48" onclick={ push }>
                        <img src="/images/push-btn.svg" />
                    </a>
                </div>
            </div>
        </div>
    }
}
