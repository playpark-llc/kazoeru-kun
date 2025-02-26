use yew::prelude::*;
use crate::components::member::{member_card::MemberCard, member_types::Member};

#[function_component]
pub fn MemberList() -> Html {
    let member_items: Vec<Member> = vec![
        Member {
            name: "PAUL/ポール".to_string(),
            role: "Business Manager".to_string(),
            description: "ITコンサルティング企業の経営をやっています。yaoyorozのよくわからない取り組みに惹かれて参加しました。ボランティアで参加しているので報酬は発生しないのですが、なぜか時間を忘れて本業に支障が出るレベルでやってる時があったりします。これってなんなんですかね？笑".to_string(),
            main_image: "/images/paul/paul.jpg".to_string(),
            sub_images: vec![],
        },
        Member {
            name: "TARO/太郎".to_string(),
            role: "Back-end Developer".to_string(),
            description: "キャンプしたり、ラーメン食べたり、歩き方の研究したり、コード書いたりしてます。".to_string(),
            main_image: "/images/taro/taro.jpg".to_string(),
            sub_images: vec![],
        },
        Member {
            name: "GORORI/ゴロリ".to_string(),
            role: "Front-end Developer".to_string(),
            description: "普段はBE・FE問わずプログラミングを主にやっています。yaoyorozのコンセプトが面白く感じて参加しました。RUSTの勉強含めて日々奮闘中！".to_string(),
            main_image: "/images/gorori/goro.jpg".to_string(),
            sub_images: vec![],
        },
        Member {
            name: "HIROKO/ひろこ".to_string(),
            role: "Designer".to_string(),
            description: "web製作会社でデザインとコーディングをしています。".to_string(),
            main_image: "/images/hiroko/hiro.jpg".to_string(),
            sub_images: vec![],
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

            <section class="bg-green-200 p-8">
                <div class="mb-8">
                    <img src="/images/about.svg" />
                </div>
                <div class="flex gap-8">
                    <a href="#" class="bg-white border border-blue-300 rounded-md shadow-md px-6 py-2">
                        {"チームコンセプト"}
                    </a>
                    <a href="#" class="bg-white border border-blue-300 rounded-md shadow-md px-6 py-2">
                        {"プロダクトコンセプト"}
                    </a>
                </div>
            </section>

            <section class="bg-green-200 p-8">
                <div class="mb-8">
                    <img src="/images/member.svg" />
                </div>
                <div class="w-full grid grid-cols-[max(240px)_max(240px)_max(240px)_max(240px)] justify-center gap-8">
                    {member_items.iter().map(|member| html! {
                        <div class="flex flex-col gap-2">
                            <MemberCard member={member.clone()} />
                            <p class="text-center mt-2">{member.name.clone()}</p>
                            <p class="text-center text-xs">{member.role.clone()}</p>
                        </div>
                    }).collect::<Html>()}
                </div>
            </section>

            <footer class="bg-pink-200 p-4 text-center">
                <span>{"@yayoyoroz"}</span>
            </footer>
        </div>
    }
}
