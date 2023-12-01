use common::{backend::BackendRun, weather, WidgetEnum, WidgetId};
use yew::prelude::*;
use yew_router::prelude::*;

use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,

    #[at("/hello-server")]
    HelloServer,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <h1>{ "Hello Frontend" }</h1> },
        Route::HelloServer => html! { <HelloServer /> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

#[function_component(HelloServer)]
fn hello_server() -> Html {
    let data = use_state(|| None);

    // Request `/api/widgets` once
    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    let result = fetch_api::<Vec<WidgetEnum>>("widgets").await;
                    data.set(Some(result));
                });
            }

            || {}
        });
    }

    match data.as_ref() {
        None => {
            html! {
                <div>{"No server response"}</div>
            }
        }
        Some(Ok(data)) => {
            html! {
                <div class="widgets">
                {
                    // construct the right component for each widget
                    data.iter().map(|widget| {
                        match widget {
                            WidgetEnum::Weather(w) => html!{<WeatherWidget definition={w.clone()} />},
                        }
                    }).collect::<Html>()
                }
                </div>
            }
        }
        Some(Err(err)) => {
            html! {
                <div>{"Error requesting data from server: "}{err}</div>
            }
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
struct WeatherWidgetProps {
    definition: common::weather::Widget,
}

#[function_component(WeatherWidget)]
fn weather_widget(props: &WeatherWidgetProps) -> Html {
    let WeatherWidgetProps { definition } = props;
    // get the most recent run here,
    // TODO: create a nicer looking UI etc

    let state = use_state(|| None);

    // Request `/api/widgets` once
    {
        let state = state.clone();
        let id = definition.id.clone();
        use_effect(move || {
            if state.is_none() {
                spawn_local(async move {
                    let result = fetch_widget_state::<weather::Output>(id, "latest").await;
                    state.set(Some(result));
                });
            }

            || {}
        });
    }

    match state.as_ref() {
        None => {
            html! {
                <div>{"No server response"}</div>
            }
        }
        Some(Ok(data)) => {
            html! {
                <div class="widgets">
                {
                    format!("{:?}", data)
                }
                </div>
            }
        }
        Some(Err(err)) => {
            html! {
                <div>{"Error requesting data from server: "}{err}</div>
            }
        }
    }
}

async fn fetch_api<T: serde::de::DeserializeOwned>(endpoint: &str) -> Result<T, String> {
    let resp = Request::get(&format!("/api/{}", endpoint))
        .send()
        .await
        .unwrap();

    if !resp.ok() {
        Err(format!(
            "Error fetching data {} ({})",
            resp.status(),
            resp.status_text()
        ))
    } else {
        // successful, get the text and try to parse it into the list of widgets
        resp.text()
            .await
            .map_err(|err| err.to_string())
            .and_then(|text| {
                serde_json::from_str::<T>(&text).map_err(|err| format!("{} content: {}", err, text))
            })
    }
}

async fn fetch_widget_state<O: serde::de::DeserializeOwned>(
    id: WidgetId,
    version: &str,
) -> Result<Option<Result<O, String>>, String> {
    let result = fetch_api::<BackendRun>(&format!("widget/{}/{}", id, version)).await;

    result.and_then(|run| {
        run.result
            .map_err(|err| format!("Backend Error: {:?}", err))
            .map(|r| {
                r.map(|text| {
                    serde_json::from_str::<O>(&text)
                        .map_err(|err| format!("{:?} content: {}", err, text))
                })
            })
    })
}
