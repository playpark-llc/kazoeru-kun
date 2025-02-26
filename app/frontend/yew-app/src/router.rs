use crate::components::{home::Home, member::member_list::MemberList,detail::Detail,push_after::PushAfter};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/push")]
    PushAfter,
    #[at("/member")]
    MemberList,
    #[at("/detail")]
    Detail,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::PushAfter => html! { <PushAfter /> },
        Route::Detail => html! { <Detail /> },
        Route::MemberList => html! { <MemberList /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}
