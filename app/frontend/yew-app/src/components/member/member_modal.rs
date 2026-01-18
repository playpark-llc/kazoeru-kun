use yew::prelude::*;
use web_sys::wasm_bindgen::JsCast;
use crate::components::member::member_types::Member;

#[derive(Properties, PartialEq, Clone)]
pub struct MemberModalProps {
    pub close_modal: Callback<MouseEvent>,
    pub member: Member,
}

#[function_component(Modal)]
pub fn modal(props: &MemberModalProps) -> Html {
    // モーダル外をクリックしたときのイベントハンドラ
    let modal_close = props.close_modal.clone();
    let on_click_outside = {
        Callback::from(move |e: MouseEvent| {
            // クリックした要素がモーダル内でなければモーダルを閉じる
            if let Some(target) = e.target() {
                if let Ok(element) = target.dyn_into::<web_sys::Element>() {
                    match element.closest(".modal") {
                        Ok(Some(_)) => {}
                        Ok(None) => {
                            modal_close.emit(e);
                        }
                        Err(err) => {
                            web_sys::console::log_1(&format!("Error: {:?}", err).into());
                        }
                    }
                }
            }
        })
    };
    html! {
        <div class="fixed inset-0 flex items-center justify-center bg-gray-200 bg-opacity-75 z-10" onclick={on_click_outside}>
            <div class="modal relative bg-blue-300 p-8 rounded-lg shadow-lg pb-[25px] w-5/6 lg:w-[800px] h-[555px] lg:grid lg:grid-rows-1 overflow-y-scroll lg:overflow-y-auto">
                // 左側の画像プレースホルダー
                <div class="lg:grid lg:grid-cols-[240px_352px] lg:grid-rows-1 lg:items-center px-4 gap-8 justify-center">
                    <div class="bg-white p-2 border border-blue-400 text-center">
                        <img src={props.member.main_image.clone()} />
                    </div>

                    // 右側のコンテンツ
                    <div class="lg:ml-8 lg:grid lg:grid-rows-[2fr_1fr] gap-2 pb-[25px]">
                        <div class="space-y-4">
                            <h2 class="text-lg font-semibold">{&props.member.name}</h2>
                            <p class="text-sm text-gray-500">{&props.member.role}</p>
                            <p class="mt-4 text-gray-600">{&props.member.description}</p>
                        </div>

                        // 三つの小さい画像プレースホルダー
                        <div>
                            <div class="flex gap-2">
                                {props.member.sub_images.iter().map(|image| html! {
                                    <div class="w-1/3 bg-white p-4 rounded-lg border border-blue-400 text-center aspect-square">
                                        <p class="text-blue-400">{image}</p>
                                    </div>
                                }).collect::<Html>()}
                            </div>
                        </div>
                    </div>
                </div>

                // 閉じるボタン
                <div class="lg:absolute lg:bottom-0 lg:right-0 flex justify-end mt-4">
                    <button onclick={&props.close_modal} class="p-2">
                        <svg xmlns="http://www.w3.org/2000/svg" width="50" height="50" viewBox="0 0 200 200">
                            <circle cx="100" cy="100" r="80" fill="none" stroke="white" stroke-width="5" />
                            <line x1="60" y1="100" x2="140" y2="100" stroke="white" stroke-width="5" />
                        </svg>
                    </button>
                </div>
            </div>
        </div>
    }
}
