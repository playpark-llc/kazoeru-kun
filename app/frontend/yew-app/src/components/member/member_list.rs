use yew::prelude::*;
use crate::components::member::{member_card::MemberCard, member_types::Member};

#[function_component]
pub fn MemberList() -> Html {
    let member_items= vec![
        Member {
            name: "PAUL/ポール".to_string(),
            role: "Business Manager".to_string(),
        },
        Member {
            name: "TARO/太郎".to_string(),
            role: "Back-end Developer".to_string(),
        },
        Member {
            name: "GORORI/ゴロリ".to_string(),
            role: "Front-end Developer".to_string(),
        },
        Member {
            name: "HIROKO/ひろこ".to_string(),
            role: "Designer".to_string(),
        },
    ];
    html! {
        <div class="min-h-screen bg-pink-200">
            <div class="flex justify-end p-4">
                <span class="text-white text-lg">{"MENU"}</span>
            </div>

            <div class="bg-blue-300 h-64 flex items-center justify-center">
                <span class="text-white text-xl">{"ここに画像が入ります"}</span>
            </div>

            <div class="bg-green-200 p-8">
                <div class="text-center mb-8">
                    <h2 class="text-3xl font-bold text-blue-700">{"ABOUT"}</h2>
                </div>
                <div class="flex justify-center space-x-4">
                    <a href="#" class="bg-white py-2 px-4 shadow-md text-blue-500 hover:bg-gray-100 rounded-lg">{"チームコンセプト"}</a>
                    <a href="#" class="bg-white py-2 px-4 shadow-md text-blue-500 hover:bg-gray-100 rounded-lg">{"プロダクトコンセプト"}</a>
                </div>
            </div>

            <div class="bg-green-200 p-8">
                <div class="text-center mb-8">
                    <h2 class="text-3xl font-bold text-blue-700">{"MEMBER"}</h2>
                </div>
                <div class="grid grid-cols-4 gap-8">
                    {member_items.iter().map(|member| html! {
                        <MemberCard name={member.name.clone()} role={member.role.clone()} />
                    }).collect::<Html>()}
                </div>
            </div>

            <footer class="bg-pink-200 p-4 text-center">
                <span>{"@yayoyoroz"}</span>
            </footer>
        </div>
    }
}
