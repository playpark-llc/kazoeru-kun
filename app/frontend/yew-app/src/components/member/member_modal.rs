use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ModalProps {
    pub close_modal: Callback<MouseEvent>,
}

#[function_component(Modal)]
pub fn modal(props: &ModalProps) -> Html {
    html! {
        <div class="fixed inset-0 flex items-center justify-center bg-gray-200 bg-opacity-75">
            <div class="bg-blue-300 p-8 rounded-lg shadow-lg w-full max-w-xl">
                // 左側の画像プレースホルダー
                <div class="flex items-center">
                    <div class="w-1/3 bg-white p-6 rounded-lg border border-blue-400 text-center">
                        <p class="text-blue-400">{"ここに画像が入ります"}</p>
                    </div>

                    // 右側のコンテンツ
                    <div class="w-2/3 ml-8">
                        <h2 class="text-lg font-semibold">{"PAUL / ポール"}</h2>
                        <p class="text-sm text-gray-500">{"Business Manager"}</p>
                        <p class="mt-4 text-gray-600">
                            {"ここにテキストが入りますここにテキストが入りますここにテキストが入りますここにテキストが入りますここにテキストが入ります"}
                        </p>

                        // 三つの小さい画像プレースホルダー
                        <div class="flex mt-6 space-x-4">
                            <div class="w-1/3 bg-white p-4 rounded-lg border border-blue-400 text-center">
                                <p class="text-blue-400">{"なにか写真があれば"}</p>
                            </div>
                            <div class="w-1/3 bg-white p-4 rounded-lg border border-blue-400 text-center">
                                <p class="text-blue-400">{"なにか写真があれば"}</p>
                            </div>
                            <div class="w-1/3 bg-white p-4 rounded-lg border border-blue-400 text-center">
                                <p class="text-blue-400">{"なにか写真があれば"}</p>
                            </div>
                        </div>
                    </div>
                </div>

                // 閉じるボタン
                <div class="flex justify-end mt-4">
                    <button onclick={&props.close_modal} class="text-blue-600 p-2">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                        </svg>
                    </button>
                </div>
            </div>
        </div>
    }
}
