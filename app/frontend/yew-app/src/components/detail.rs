use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use crate::env;
use chrono::{Local, Duration};

fn format_number(num: u32) -> String {
    let s = num.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, ch);
    }
    result
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct SessionCounterSummary {
    sum_count: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ApiResponse {
    message: String,
    result: SessionCounterSummary,
}

#[function_component]
pub fn Detail() -> Html {
    let count_data = use_state(|| None::<u32>);
    let is_loading = use_state(|| true);
    let error_message = use_state(|| None::<String>);
    const GOAL: u32 = 10000;

    {
        let count_data = count_data.clone();
        let is_loading = is_loading.clone();
        let error_message = error_message.clone();
        
        use_effect_with((), move |_| {
            spawn_local(async move {
                // 昨日までの日付範囲を計算
                let today = Local::now();
                let yesterday = today - Duration::days(1);
                let start_date = "2020-01-01";
                let end_date = yesterday.format("%Y-%m-%d").to_string();
                
                let client = reqwest::Client::new();
                let url = format!(
                    "{}/count_summary?start_date={}&end_date={}",
                    env::APP_API_ROOT,
                    start_date,
                    end_date
                );
                
                match client.get(&url).send().await {
                    Ok(response) => {
                        match response.json::<ApiResponse>().await {
                            Ok(data) => {
                                // sum_countをf64として解析してからu32に変換
                                if let Some(sum_count_str) = data.result.sum_count {
                                    if let Ok(count) = sum_count_str.parse::<f64>() {
                                        count_data.set(Some(count as u32));
                                    } else {
                                        count_data.set(Some(0));
                                    }
                                } else {
                                    count_data.set(Some(0));
                                }
                            }
                            Err(e) => {
                                error_message.set(Some(format!("データの解析に失敗しました: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("データの取得に失敗しました: {}", e)));
                    }
                }
                is_loading.set(false);
            });
            || ()
        });
    }
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
                <div class="w-1/2 h-2/3 px-1 bg-[#F5BED3] rounded-lg relative">
                    <div class="flex justify-end items-center py-2 pr-4 text-white h-[60px]">
                        <span class="text-[18px] tracking-widest">{"MENU"}</span>
                    </div>
                    <div class="bg-primary-check flex flex-col justify-center items-center absolute inset-0 mt-[60px] mb-[50px] mx-1">
                        <div class="flex justify-center">{"push me!!"}</div>
                        <div class="m-4 w-2/3 text-center relative z-10 aspect-square">
                            <div class="h-full w-full border-2 border-[#6184C6] bg-white rounded flex flex-col p-4">
                                {if *is_loading {
                                    html! {
                                        <div class="text-center text-xl">
                                            <p class="text-[#6184C6]">{ "読み込み中..." }</p>
                                        </div>
                                    }
                                } else if let Some(error) = &*error_message {
                                    html! {
                                        <div class="text-center text-sm text-red-500">
                                            <p>{ error }</p>
                                        </div>
                                    }
                                } else {
                                    let count = (*count_data).unwrap_or(0);
                                    let remaining = if count >= GOAL { 0 } else { GOAL - count };
                                    let formatted_count = format_number(count);
                                    let formatted_remaining = format_number(remaining);
                                    
                                    html! {
                                        <>
                                            <div class="text-center mb-2 text-xl">
                                                <p>{"昨日までの"}</p>
                                                <p>{"つんつん数"}</p>
                                                <div class="mt-1">
                                                    <span class="border-b-dashed border-b-pink-300 border-b">
                                                        <span class="text-2xl font-bold text-[#6184C6]">{ formatted_count }</span>
                                                        <span class="text-base">{"回"}</span>
                                                    </span>
                                                </div>
                                            </div>
                                            
                                            <div class="text-center text-2xl">
                                                {if remaining > 0 {
                                                    html! {
                                                        <>
                                                            <p>{"目標まであと"}</p>
                                                            <p class="font-bold text-[28px]">
                                                                <span class="text-[#6184C6] tracking-widest">{ formatted_remaining }</span>
                                                                <span class="text-xl">{"つんつん！"}</span>
                                                            </p>
                                                        </>
                                                    }
                                                } else {
                                                    html! {
                                                        <>
                                                            <p class="text-[#6184C6] font-bold">{"目標達成！"}</p>
                                                            <p class="text-xl">{"おめでとう！"}</p>
                                                        </>
                                                    }
                                                }}
                                            </div>
                                        </>
                                    }
                                }}
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
