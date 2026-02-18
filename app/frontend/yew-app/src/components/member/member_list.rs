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
        <div class="min-h-screen bg-pink-200 w-screen font-DotGothic">
            <div class="flex justify-end p-4">
                <span class="text-white text-lg">{"MENU"}</span>
            </div>

            <div class="bg-blue-300 h-[600px] flex items-center justify-center">
                <img class="p-10" src="/images/main-logo.svg" alt="" />
            </div>

            <section class="bg-primary-check py-8 md:pl-8 xl:pr-8 w-full">
                <div class="py-8 px-4 md:px-8">
                    <div class="mb-8">
                        <img src="/images/about.svg" />
                    </div>
                    <div class="flex flex-col md:flex-row gap-8">
                        <a href="">
                            <div class="relative max-w-xs border-[#6184C6]">
                                <button class="w-full flex items-center justify-between rounded bg-white px-5 py-3 text-[#6184C6] shadow-md relative z-10 gap-4">
                                    <span class="text-base">{"チームコンセプト"}</span>
                                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"></path>
                                        <polyline points="15 3 21 3 21 9"></polyline>
                                        <line x1="10" y1="14" x2="21" y2="3"></line>
                                    </svg>
                                </button>
                                <div class="absolute bottom-0 right-0 w-full h-full bg-[#6184C6] rounded translate-x-1 translate-y-1"></div>
                            </div>
                        </a>
                        <a href="">
                            <div class="relative max-w-xs border-[#6184C6]">
                                <button class="w-full flex items-center justify-between rounded bg-white px-5 py-3 text-[#6184C6] shadow-md relative z-10 gap-4">
                                    <span class="text-base">{"プロダクトコンセプト"}</span>
                                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"></path>
                                        <polyline points="15 3 21 3 21 9"></polyline>
                                        <line x1="10" y1="14" x2="21" y2="3"></line>
                                    </svg>
                                </button>
                                <div class="absolute bottom-0 right-0 w-full h-full bg-[#6184C6] rounded translate-x-1 translate-y-1"></div>
                            </div>
                        </a>
                    </div>
                </div>

                <div class="py-8 md:pl-8 xl:pr-8">
                    <div class="mb-8 pl-4 md:pl-0">
                        <img src="/images/member.svg" />
                    </div>
                    <div class="overflow-x-scroll md:w-full flex gap-8 overflow-y-hidden pb-4 px-4 md:px-0">
                        {member_items.iter().map(|member| html! {
                            <div class="flex flex-col gap-2 min-w-[262px]">
                                <MemberCard member={member.clone()} />
                                <p class="text-center mt-2 text-[#6184C6]">{member.name.clone()}</p>
                                <p class="text-center text-xs text-[#6184C6]">{member.role.clone()}</p>
                            </div>
                        }).collect::<Html>()}
                    </div>
                </div>
            </section>

            <footer class="bg-pink-200 p-4 text-center text-white">
                <span>{"@yaoyoroz"}</span>
            </footer>
        </div>
    }
}
