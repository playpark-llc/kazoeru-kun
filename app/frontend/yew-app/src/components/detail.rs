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
                            <a href="https://docs.google.com/forms/d/e/1FAIpQLSdLGxR5WyiUXXVTXl_RLF-NvEDRrMtLdGftFRD6urhvRtNSuQ/viewform?usp=dialog" class="block" about="_blank">
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
                        <span class="text-[18px] tracking-widest lg:hidden">{"MENU"}</span>
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
                            <div class="relative flex items-center hover:-translate-y-1 active:translate-y-0">
                                <a class="w-48 cursor-pointer" onclick={ push }>
                                    <img src={ status_state.image_url() } />
                                </a>
                                <div class="absolute -top-10 right-0">
                                    <div class="relative w-[100px]">
                                        <svg class="w-full" width="113" height="99" viewBox="0 0 113 99" fill="none" xmlns="http://www.w3.org/2000/svg">
                                            <path d="M61.0762 80.8855C61.0762 80.8855 75.8518 81.6249 85.7798 77.5071C93.5138 74.3008 101.432 70.4635 106.077 63.2796C108.602 59.3785 110.146 54.8591 111.066 50.2505C112.739 41.8428 112.291 32.753 108.498 25.1229C104.674 17.4291 97.724 11.756 90.0513 8.38393C82.3785 5.01829 73.9948 3.76892 65.6847 3.2781C54.7639 2.62792 43.6286 3.26535 33.2655 6.90509C22.9024 10.5448 13.3359 17.4419 7.77134 27.2328C2.20675 37.0238 1.14654 49.8553 6.42309 59.812C10.1798 66.9002 16.7372 72.0379 23.8339 75.359C30.9306 78.68 38.4808 81.2871 47.1524 81.0576C47.1524 81.0576 44.4192 94.705 37.0406 97.5416C37.0406 97.5416 55.0458 96.3433 61.0762 80.8855Z" fill="#6184C6" stroke="#6184C6" stroke-width="2"/>
                                            <path d="M59.0945 78.8313C59.0945 78.8313 73.8701 79.5707 83.7981 75.4529C91.5321 72.2466 99.45 68.4093 104.095 61.2254C106.62 57.3243 108.165 52.8049 109.084 48.1963C110.757 39.7886 110.31 30.6988 106.516 23.0687C102.692 15.3749 95.7423 9.70175 88.0696 6.32973C80.3968 2.96409 72.0132 1.71472 63.703 1.2239C52.7822 0.573719 41.6469 1.21115 31.2838 4.85089C20.9207 8.49063 11.3542 15.3877 5.78965 25.1786C0.225061 34.9696 -0.835153 47.8011 4.4414 57.7578C8.19811 64.846 14.7555 69.9837 21.8522 73.3048C28.9489 76.6258 36.4991 79.2329 45.1708 79.0034C45.1708 79.0034 42.4375 92.6508 35.0589 95.4874C35.0589 95.4874 53.0641 94.2891 59.0945 78.8313Z" fill="white" stroke="#6184C6" stroke-width="2"/>
                                        </svg>

                                        <div class="absolute inset-0 flex items-center justify-center">
                                            <p class="text-[45px] font-Pomeranian text-[#6184C6] pb-3.5 -tracking-[18px] pr-4 text-center">{ *daily_count }</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
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
