use yew::prelude::*;

#[function_component]
pub fn Detail() -> Html {
    html! {
        <div class="w-full flex">
            <div class="w-1/2 p-4 bg-green-100">
                <div class="flex justify-center"><img src="images/menu.svg" /></div>
                <div class="mt-4 space-y-3">
                    <button class="w-full bg-white border-2 border-blue-500 shadow-md py-3 px-4 rounded-lg flex justify-between items-center">
                        {"アプリについて"}
                    </button>
                    <button class="w-full bg-white border-2 border-blue-500 shadow-md py-3 px-4 rounded-lg flex justify-between items-center">
                        {"運営チーム"}
                    </button>
                    <button class="w-full bg-white border-2 border-blue-500 shadow-md py-3 px-4 rounded-lg flex justify-between items-center">
                        {"お問い合わせ"}
                    </button>
                </div>

                <div class="mt-6 text-center">
                    <div class="flex justify-center"><img src="images/news.svg" /></div>
                    <div class="bg-white border-2 border-blue-500 shadow-md p-4 rounded-lg mt-2">
                        <h3 class="text-xl font-bold">{"no+e"}</h3>
                        <p class="text-blue-500 text-sm">{"チーム yaoyoroz"}</p>
                        <p class="text-lg font-bold text-gray-800">{"研究開発テーマ"}</p>
                        <a href="#" class="text-blue-500 text-sm mt-2 block">{"記事を読む"}</a>
                    </div>
                </div>

                <div class="mt-6 flex justify-center space-x-4">
                    <button class="w-12 h-12 bg-white border-2 border-blue-500 shadow-md rounded-full flex items-center justify-center font-bold">
                        <img src="images/icon-note.svg" />
                    </button>
                    <button class="w-12 h-12 bg-white border-2 border-blue-500 shadow-md rounded-full flex items-center justify-center font-bold">
                        <img src="images/icon-x.svg" />
                    </button>
                </div>
            </div>

            <div class="w-1/2 p-4 bg-blue-200 relative">
                <div class="absolute top-2 right-2 bg-white p-2 rounded-full shadow-md transform rotate-[-15deg]">
                    <p class="text-blue-500 font-bold text-sm">{"うざい？"}</p>
                </div>
                <div class="p-6 bg-pink-300 rounded-lg shadow-md">
                    <h2 class="text-xl font-bold text-blue-500 text-center drop-shadow-md">{"PUSH ME!"}</h2>
                    <div class="mt-4 p-4 bg-green-200 border-2 border-blue-500 shadow-md rounded-lg text-center">
                        <p class="text-lg font-bold font-DotGothic">{"昨日までのつんつん数"}</p>
                        <p class="text-xl text-blue-500 font-bold">{"1,234回"}</p>
                        <p class="mt-2 text-lg font-bold">{"目標まであと"}</p>
                        <p class="text-xl text-blue-500 font-bold">{"100つんつん!"}</p>
                    </div>
                </div>
                <div class="absolute bottom-2 right-4 text-sm text-gray-600">{"@yaoyoroz"}</div>
            </div>
        </div>
    }
}
