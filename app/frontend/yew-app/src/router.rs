use crate::components::{home::Home, member::member_list::MemberList};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/push")]
    PushAfter,
    #[at("/watch")]
    Watch,
    #[at("/member")]
    MemberList,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::PushAfter => html! { <PushAfter /> },
        Route::Watch => html! { <Watch /> },
        Route::MemberList => html! { <MemberList /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component]
fn PushAfter() -> Html {
    html! {
        <div class="w-screen h-screen relative" style="
            height: 100dvh;
            background-image: repeating-linear-gradient(
                90deg,
                #fff ,
                #fff 1px,
                transparent 1px,
                transparent 20px
            ), repeating-linear-gradient(
                0deg,
                #fff ,
                #fff 1px,
                #C0E2D1 1px,
                #C0E2D1 20px
            );">
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

#[function_component]
fn Watch() -> Html {
    html! {
        <div class="flex">
            <div class="w-full h-full relative" style="
                height: 100dvh;
                background-image: repeating-linear-gradient(
                    90deg,
                    #fff ,
                    #fff 1px,
                    transparent 1px,
                    transparent 20px
                ), repeating-linear-gradient(
                    0deg,
                    #fff ,
                    #fff 1px,
                    #C0E2D1 1px,
                    #C0E2D1 20px
                );">
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

            <div></div>
            <div>
                
            </div>
        </div>
    }
}
