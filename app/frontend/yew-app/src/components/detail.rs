use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo::console::error;
use web_sys::MouseEvent;
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
pub fn Detail() -> Html {
    // 状態を定義
    let count = use_state(|| 0u32);
    {
        let count = count.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match ApiClient::get_count_history().await {
                    Ok(count_data) => {
                        count.set(count_data);
                    }
                    Err(e) => {
                        error!("Failed to get status history: {}", e.to_string());
                    }
                }
            });
            || ()
        });
    }

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

    let is_menu_open = use_state(|| false);
    let toggle_menu = {
        let is_menu_open = is_menu_open.clone();
        Callback::from(move |_: MouseEvent| {
            is_menu_open.set(!*is_menu_open);
        })
    };

    // sessionStorageから値を読み込み
    let daily_count = use_state(|| 0u32);
    {
        let daily_count = daily_count.clone();
        use_effect_with((), move |_| {
            match get_session_storage() {
                Ok(storage) => {
                    match storage.get_item("kazoerukun_count") {
                        Ok(Some(value_string)) => {
                            
                            // JSON文字列をデシリアライズ
                            match serde_json::from_str::<u32>(&value_string) {
                                Ok(count) => {
                                    daily_count.set(count);
                                }
                                Err(_e) => {
                                    daily_count.set(0);
                                }
                            }
                        }
                        Ok(None) => {
                            daily_count.set(0);
                        }
                        Err(_e) => {
                            daily_count.set(0);
                        }
                    }
                }
                Err(_e) => {
                    daily_count.set(0);
                }
            }
            || ()
        });
    }

    let push = {
        let daily_count = daily_count.clone();
        move |_| {
            let daily_count = daily_count.clone();
            let storage = get_session_storage().unwrap();
            spawn_local(async move {
                match ApiClient::send_session().await {
                    Ok(click_count) => {
                        let json_string = serde_json::to_string(&click_count).unwrap();
                        storage.set_item("kazoerukun_count", &json_string).unwrap();
                        match serde_json::from_str::<u32>(&json_string) {
                            Ok(count) => {
                                daily_count.set(count);
                            }
                            Err(_e) => {
                                daily_count.set(0);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to send session: {}", e.to_string());
                    }
                }
            });
        }
    };

    html! {
        <div class="w-full h-screen lg:flex font-DotGothic relative">
            <div
                class={format!("w-full h-full absolute inset-0 z-20 lg:static lg:w-1/2 p-4 bg-[#F5BED3] lg:bg-[#C0E2D1] justify-center items-center text-blue-500 {} lg:translate-x-0",
                    if *is_menu_open { "translate-x-0" } else { "-translate-x-full" }
                )}
            >
                <div class="flex relative w-full h-full items-center justify-center">
                    <div class="absolute -top-4 -left-4 lg:hidden w-4/10">
                        <a href="/push">
                            <img src="images/uzai-right.svg" />
                        </a>
                    </div>
                    <div class="absolute top-0 right-0 text-white lg:hidden" onclick={ toggle_menu.clone() }>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
                        </svg>
                    </div>
                    <div class="w-5/6 lg:w-1/2">
                        <div class="flex justify-center">
                            <img src="images/menu.svg" />
                        </div>
                        <div class="mt-4 my-16 space-y-3">
                            // <button class="w-full bg-white border-2 border-blue-500 shadow-md py-3 px-4 rounded-lg flex justify-between items-center">
                            //     {"アプリについて"}
                            // </button>
                            <a href="/member" class="block">
                                <div class="relative w-full">
                                    <button class="w-full flex items-center justify-between rounded bg-white px-5 py-3 text-[#6184C6] shadow-md relative z-10 gap-4 border border-[#6184C6]">
                                        <div class="size-6"></div>
                                        <p class="text-base w-full text-center">{"運営チーム"}</p>
                                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                                            <path stroke-linecap="round" stroke-linejoin="round" d="m8.25 4.5 7.5 7.5-7.5 7.5" />
                                        </svg>
                                    </button>
                                    <div class="absolute bottom-0 right-0 w-full h-full bg-[#6184C6] rounded translate-x-1 translate-y-1"></div>
                                </div>
                            </a>
                            <a href="https://docs.google.com/forms/d/e/1FAIpQLSdLGxR5WyiUXXVTXl_RLF-NvEDRrMtLdGftFRD6urhvRtNSuQ/viewform?usp=dialog" class="block">
                                <div class="relative w-full">
                                    <button class="w-full flex items-center justify-between rounded bg-white px-5 py-3 text-[#6184C6] shadow-md relative z-10 gap-4 border border-[#6184C6]">
                                        <div class="size-6"></div>
                                        <p class="text-base w-full text-center">{"お問い合わせ"}</p>
                                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                                            <path stroke-linecap="round" stroke-linejoin="round" d="m8.25 4.5 7.5 7.5-7.5 7.5" />
                                        </svg>
                                    </button>
                                    <div class="absolute bottom-0 right-0 w-full h-full bg-[#6184C6] rounded translate-x-1 translate-y-1"></div>
                                </div>
                            </a>
                        </div>

                        <div class="mt-6 text-center">
                            <div class="flex justify-center">
                                <img src="images/news.svg" />
                            </div>
                            <div class="relative w-full">
                                <div class="w-full rounded bg-white text-[#6184C6] shadow-md z-10 p-4 mt-2 shadow-lg relative border border-[#6184C6]">
                                    <div class="flex justify-center pb-4"><img src="images/note-logo.svg" /></div>
                                    <p class="text-[22px] font-ZenKakuGothic text-[#6184C6] pb-2">{"チームyaoyoroz"}</p>
                                    <p class="text-[32px] font-bold font-ZenKakuGothic text-[#6184C6] pb-4">{"研究開発テーマ"}</p>
                                    <div class="flex justify-end">
                                        <a href="https://note.com/yaoyoroz_/" class="text-[#6184C6] text-sm mt-2 block" target="_blank">
                                            <div class="flex justify-center items-center gap-1">
                                                <span>{"記事を読む"}</span>
                                                <div class="bg-[#6184C6] text-white flex items-center justify-center rounded-full p-1">
                                                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-3">
                                                        <path stroke-linecap="round" stroke-linejoin="round" d="m8.25 4.5 7.5 7.5-7.5 7.5" />
                                                    </svg>
                                                </div>
                                            </div>
                                        </a>
                                    </div>
                                </div>
                                <div class="absolute bottom-0 right-0 w-full h-full bg-[#6184C6] rounded translate-x-1 translate-y-1 shadow-lg"></div>
                            </div>
                        </div>

                        <div class="mt-6 flex justify-center space-x-4">
                            <div class="relative">
                                <button class="w-12 h-12 bg-white border border-[#6184C6] z-10 shadow-md relative rounded-full flex items-center justify-center font-bold">
                                    <a href="https://note.com/yaoyoroz_/" target="_blank"><img src="images/icon-note.svg" /></a>
                                </button>
                                <div class="absolute bottom-0 right-0 w-full h-full bg-[#6184C6] rounded-full translate-x-0.5 translate-y-0.5 shadow-lg"></div>
                            </div>
                            <div class="relative">
                                <button class="w-12 h-12 bg-white border border-[#6184C6] z-10 shadow-md relative rounded-full flex items-center justify-center font-bold">
                                    <a href="https://twitter.com/yaoyorozu_team" target="_blank"><img src="images/icon-x.svg" /></a>
                                </button>
                                <div class="absolute bottom-0 right-0 w-full h-full bg-[#6184C6] rounded-full translate-x-0.5 translate-y-0.5 shadow-lg"></div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="bg-[#F5BED3] w-[55px] relative overflow-hidden hidden lg:block">
                <div class="absolute inline-block -rotate-90 text-white inset-0 flex flex-nowrap items-center justify-center text-2xl">
                    <span class="px-4">{"KAZOERUKUN!!"}</span>
                    <span class="px-4">{"KAZOERUKUN!!"}</span>
                    <span class="px-4">{"KAZOERUKUN!!"}</span>
                    <span class="px-4">{"KAZOERUKUN!!"}</span>
                    <span class="px-4">{"KAZOERUKUN!!"}</span>
                    <span class="px-4">{"KAZOERUKUN!!"}</span>
                    <span class="px-4">{"KAZOERUKUN!!"}</span>
                    <span class="px-4">{"KAZOERUKUN!!"}</span>
                    <span class="px-4">{"KAZOERUKUN!!"}</span>
                </div>
            </div>

            <div class="w-full lg:w-1/2 lg:p-4 bg-[#8CB5E1] relative flex justify-center items-center">
                <div class="absolute top-0 right-0 hidden lg:block">
                    <a href="/push">
                        <img src="images/uzai.svg" />
                    </a>
                </div>
                <div class="w-full lg:w-1/2 h-screen lg:h-5/6 lg:px-1 bg-[#F5BED3] lg:rounded-lg relative">
                    <div class="justify-end items-center py-2 pr-4 text-white h-[60px] hidden lg:flex">
                        <span class="text-[18px] tracking-widest">{"MENU"}</span>
                    </div>
                    <div class="flex justify-end items-center py-2 pr-4 text-white h-[60px] lg:hidden" onclick={ toggle_menu.clone() }>
                        <span class="text-[18px] tracking-widest">{"MENU"}</span>
                    </div>
                    <div class="bg-primary-check flex flex-col justify-center items-center absolute inset-0 mt-[60px] mb-[50px] lg:mx-1">
                        <div class="flex flex-col justify-center items-center space-y-9">
                            <div class="flex flex-col items-center space-y-4">
                                <img src="/images/sun.svg" />
                                <img src="/images/main-text.svg" class="w-full"/>
                            </div>
                            <a class="w-48 hover:-translate-y-1 active:translate-y-0 cursor-pointer" onclick={ push }>
                                <img src={ status_state.image_url() } />
                            </a>
                        </div>
                        <div class="m-4 w-[232px] lg:w-2/3 text-center relative z-10 aspect-square">
                            <div class="h-full w-full border-2 border-[#6184C6] bg-white rounded flex flex-col p-4">
                                <div class="text-center mb-2 text-xl">
                                    <p>{"昨日までの"}</p>
                                    <p>{"つんつん数"}</p>
                                    <div class="mt-1">
                                        <span class="border-b-dashed border-b-pink-300 border-b">
                                            <span class="text-2xl font-bold text-[#6184C6]">{ *count }</span>
                                            <span class="text-base">{"回"}</span>
                                        </span>
                                    </div>
                                    <div class="mt-1">
                                        <span class="border-b-dashed border-b-pink-300 border-b">
                                            <span class="text-2xl font-bold text-[#6184C6]">{ *daily_count }</span>
                                            <span class="text-base">{"回"}</span>
                                        </span>
                                    </div>
                                </div>
                                
                                <div class="text-center text-2xl">
                                    <p>{"目標まであと"}</p>
                                    <p class="font-bold text-[28px]">
                                        <span class="text-[#6184C6] tracking-widest">{ status_state.current_target() - *count }</span>
                                        <span class="text-xl">{"つんつん！"}</span>
                                    </p>
                                </div>
                            </div>

                            <div class="absolute inset-0 bg-[#6184C6] rounded transform translate-x-1 translate-y-1 -z-10"></div>
                        </div>
                    </div>
                    <div class="absolute bottom-0 right-0 text-sm text-gray-600 h-[50px] w-full text-white flex justify-center items-center">
                        {"@yaoyoroz"}
                    </div>
                </div>
            </div>
        </div>
    }
}
