use yew::prelude::*;
use crate::components::member::member_modal::Modal;
use crate::components::member::member_types::Member;

#[derive(Properties, PartialEq, Clone)]
pub struct MemberCardProps {
    pub member: Member
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
        <div class="border-2 border-[#6184C6] p-2 bg-white flex flex-col items-center shadow-lg relative">
            <div class="w-full bg-blue-200 flex items-center justify-center">
                <img src={props.member.main_image.clone()} />
            </div>
            <div onclick={open_modal} class="mt-4 bg-[#6184C6] flex justify-center items-center text-white rounded-full w-8 h-8 absolute bottom-4 right-5">
                <p class="">{"＋"}</p>
            </div>
            // モーダル表示部分
            if *is_modal_open {
                <Modal {close_modal} member={props.member.clone()} />
            }
        </div>
    }
}