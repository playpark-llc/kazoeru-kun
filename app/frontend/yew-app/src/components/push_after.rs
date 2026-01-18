use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo::console::error;
use web_sys::window;
use web_sys::Storage;

use crate::models::status::Status;
use crate::api_client::ApiClient;

fn get_session_storage() -> Result<Storage, String> {
    window()
        .ok_or("Window not available")?
        .session_storage()
        .map_err(|_| "Failed to access sessionStorage")?
        .ok_or("SessionStorage not available".to_string())
}

#[function_component]
pub fn PushAfter() -> Html {
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

    let push = {
        move |_| {
            let storage = get_session_storage().unwrap();
            spawn_local(async move {
                match ApiClient::send_session().await {
                    Ok(click_count) => {
                        let json_string = serde_json::to_string(&click_count).unwrap();
                        storage.set_item("kazoerukun_count", &json_string).unwrap();
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
            <div class="mb-12 absolute w-full flex justify-center bottom-0">
                <a href="detail">
                    <img src="/images/watch.svg" />
                </a>
            </div>
        </div>
    }
}