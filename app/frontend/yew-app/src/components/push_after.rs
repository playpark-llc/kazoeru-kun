use yew::prelude::*;

#[function_component]
pub fn PushAfter() -> Html {
    html! {
        <div class="w-screen h-screen relative bg-primary-check">
            <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-[72.2%]">
                <div class="flex flex-col justify-center items-center space-y-9">
                    <img src="/images/one-more.svg" />
                    <div class="w-48">
                        <img src="/images/push-btn.svg" />
                    </div>
                </div>
            </div>
            <div class="mb-12 absolute w-full flex justify-center bottom-0">
                <img src="/images/watch.svg" />
            </div>
        </div>
    }
}