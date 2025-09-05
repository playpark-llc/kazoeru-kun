use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::env;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct ApiResponse {
    message: String,
    result: ResultData,
}
#[derive(Deserialize, Debug)]
struct ResultData {
    sum_count: String,
    // sum_count: 0u32,
}

#[function_component]
pub fn Detail() -> Html {
    let target_count = use_state(|| 100_000u32);
    // 状態を定義
    let count = use_state(|| 0u32);
    {
        let count = count.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let client = reqwest::Client::new();
                let _req = client
                    .get(format!("{}/{}", env::APP_API_ROOT, "session_counter?start_date=2025-08-19&end_date=2025-08-21"))
                    .send()
                    .await
                    .expect("failed to send request");
                let _data: ApiResponse = _req.json().await.expect("failed to parse JSON");

                // 状態を更新
                count.set(_data.result.sum_count.parse().unwrap());
            });
            || ();

            spawn_local(async move {
                let client = reqwest::Client::new();
                let _req = client
                    .get(format!("{}/{}", env::APP_API_ROOT, "status_history"))
                    .send()
                    .await
                    .expect("failed to send request");
            });
            || ()
        });
    }

    let push = {
        move |_| {
            spawn_local(async move {
                let client = reqwest::Client::new();
                client.post(format!("{}/{}", env::APP_API_ROOT, "session"))
                    .send()
                    .await
                    .expect("failed to send request");
            });
        }
    };

    html! {
        <div class="w-full h-screen flex font-DotGothic">
            <div class="w-1/2 p-4 bg-green-100 flex justify-center items-center text-blue-500">
                <div class="w-1/2">
                    <div class="flex justify-center">
                        <img src="images/menu.svg" />
                    </div>
                    <div class="mt-4 space-y-3">
                        // <button class="w-full bg-white border-2 border-blue-500 shadow-md py-3 px-4 rounded-lg flex justify-between items-center">
                        //     {"アプリについて"}
                        // </button>
                        <a href="/member" class="block">
                            <div class="relative w-full">
                                <button class="w-full flex items-center justify-between rounded bg-white px-5 py-3 text-[#6184C6] shadow-md relative z-10 gap-4 border border-[#6184C6]">
                                    <p class="text-base w-full text-center">{"運営チーム"}</p>
                                    <div class="absolute right-5 w-4 h-4 border-4 border-gray-600 border-b-transparent border-l-transparent rotate-45"></div>
                                </button>
                                <div class="absolute bottom-0 right-0 w-full h-full bg-[#6184C6] rounded translate-x-1 translate-y-1"></div>
                            </div>
                        </a>
                        <a href="https://docs.google.com/forms/d/e/1FAIpQLSdLGxR5WyiUXXVTXl_RLF-NvEDRrMtLdGftFRD6urhvRtNSuQ/viewform?usp=dialog" class="block">
                            <div class="relative w-full">
                                <button class="w-full flex items-center justify-between rounded bg-white px-5 py-3 text-[#6184C6] shadow-md relative z-10 gap-4 border border-[#6184C6]">
                                    <p class="text-base w-full text-center">{"お問い合わせ"}</p>
                                    <div class="absolute right-5 w-4 h-4 border-4 border-gray-600 border-b-transparent border-l-transparent rotate-45"></div>
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
                                <div class="flex justify-center"><img src="images/note-logo.svg" /></div>
                                <p class="text-sm text-[#6184C6]">{"チーム yaoyoroz"}</p>
                                <p class="text-lg font-bold text-[#6184C6]">{"研究開発テーマ"}</p>
                                <div class="flex justify-end"><a href="https://note.com/yaoyoroz_/" class="text-[#6184C6] text-sm mt-2 block" target="_blank">{"記事を読む"}</a></div>
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

            <div class="bg-[#F5BED3] w-[55px] relative overflow-hidden">
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

            <div class="w-1/2 p-4 bg-[#8CB5E1] relative flex justify-center items-center">
                <div class="absolute top-0 right-0">
                    <a href="/push">
                        <img src="images/uzai.svg" />
                    </a>
                </div>
                <div class="w-1/2 h-5/6 px-1 bg-[#F5BED3] rounded-lg relative">
                    <div class="flex justify-end items-center py-2 pr-4 text-white h-[60px]">
                        <span class="text-[18px] tracking-widest">{"MENU"}</span>
                    </div>
                    <div class="bg-primary-check flex flex-col justify-center items-center absolute inset-0 mt-[60px] mb-[50px] mx-1">
                        <div class="flex flex-col justify-center items-center space-y-9">
                            <div class="flex flex-col items-center space-y-4">
                                <img src="/images/sun.svg" />
                                <img src="/images/main-text.svg" class="w-full"/>
                            </div>
                            <a class="w-48 hover:-translate-y-1 cursor-pointer" onclick={ push }>
                                <img src="/images/status/dirty.png" />
                            </a>
                        </div>
                        <div class="m-4 w-2/3 text-center relative z-10 aspect-square">
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
                                        <span class="text-[#6184C6] tracking-widest">{ *target_count - *count }</span>
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
