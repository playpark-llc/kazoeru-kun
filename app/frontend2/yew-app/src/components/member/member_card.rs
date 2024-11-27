use yew::prelude::*;
use crate::components::member::member_modal::Modal;

#[derive(Properties, PartialEq)]
pub struct MemberCardProps {
    pub name: String,
    pub role: String,
}

#[function_component(MemberCard)]
pub fn member_card(props: &MemberCardProps) -> Html {
    // モーダル表示・非表示の状態管理
    let is_modal_open = use_state(|| false);

    // モーダルを開く関数
    let open_modal = {
        let is_modal_open = is_modal_open.clone();
        Callback::from(move |_| is_modal_open.set(true))
    };

    // モーダルを閉じる関数
    let close_modal = {
        let is_modal_open = is_modal_open.clone();
        Callback::from(move |_| is_modal_open.set(false))
    };

    html! {
        <div class="border-2 border-blue-400 p-4 bg-white flex flex-col items-center shadow-lg">
            <div class="h-32 w-full bg-blue-200 flex items-center justify-center">
                <span class="text-gray-500">{"ここに画像が入ります"}</span>
            </div>
            <h3 class="mt-4 text-blue-500 text-lg font-semibold">{&props.name}</h3>
            <p class="text-gray-500">{&props.role}</p>
            <button onclick={open_modal} class="mt-4 bg-blue-500 text-white rounded-full w-8 h-8 flex items-center justify-center">
                <span>{"+"}</span>
            </button>
            // モーダル表示部分
            if *is_modal_open {
                <Modal {close_modal} />
            }
        </div>
    }
}