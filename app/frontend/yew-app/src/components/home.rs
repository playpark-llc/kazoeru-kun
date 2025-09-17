use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo::console::error;

use crate::router::Route;
use crate::models::status::Status;
use crate::api_client::ApiClient;

#[function_component]
pub fn Home() -> Html {
    let status_state = use_state(|| Status::Clean);
    
    // ステータス履歴の取得
    {
        let status_state = status_state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match ApiClient::get_status_history().await {
                    Ok(status_str) => {
                        let status = Status::from(status_str.as_str());
                        status_state.set(status);
                    }
                    Err(e) => {
                        error!("Failed to get status history: {}", e.to_string());
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
                match ApiClient::send_session().await {
                    Ok(_) => {
                        navigator.push(&Route::PushAfter);
                    }
                    Err(e) => {
                        error!("Failed to send session: {}", e.to_string());
                    }
                }
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
                        <img src={ status_state.image_url() } />
                    </a>
                </div>
            </div>
        </div>
    }
}
