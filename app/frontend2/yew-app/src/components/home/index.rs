use yew::prelude::*;
use yew_router::prelude::*;
use crate::router::Route;
use wasm_bindgen_futures::spawn_local;

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
                    .post("http://localhost:8881/session")
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
        <div class="w-screen h-screen" style="height: 100vh; background-image:
            repeating-linear-gradient(
            90deg,
            #fff ,
            #fff 1px,
            transparent 1px,
            transparent 20px
            ),
            repeating-linear-gradient(
            0deg,
            #fff ,
            #fff 1px,
            #C0E2D1 1px,
            #C0E2D1 20px
            );">
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
