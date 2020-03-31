pub mod graph;
mod handlers;

use crate::flow;
use crate::flow::Node;
use crate::import::*;
use mcmf::Flow;
use tokio::sync::watch::Receiver;
use warp::Filter;

type SchedulerSubscription = Receiver<Vec<Flow<Node>>>;
type Scheduler = Arc<Mutex<flow::Scheduler>>;

pub async fn serve(scheduler: Scheduler) {
    let flow_subscription = scheduler.lock().await.subscribe();
    let routes = get_schedule_graph()
        .or(get_index())
        .or(get_api_schedule_graph(flow_subscription))
        .or(get_server(scheduler.clone()))
        .or(post_server(scheduler.clone()));
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

fn get_schedule_graph() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::get().and(warp::path!("schedule" / "graph")).and_then(handlers::get_graph_html)
}

fn get_api_schedule_graph(
    graph: SchedulerSubscription,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let graph = warp::any().map(move || graph.clone());
    warp::path!("api" / "schedule" / "graph").and(warp::ws()).and(graph).map(
        |ws: warp::ws::Ws, graph| ws.on_upgrade(move |socket| handlers::graphflow(socket, graph)),
    )
}

fn get_index() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(warp::path::end()).and_then(handlers::get_index)
}

fn get_server(
    scheduler: Scheduler,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let scheduler = warp::any().map(move || scheduler.clone());
    warp::get()
        .and(warp::path!("schedule" / "server"))
        .and(scheduler)
        .and_then(|scheduler| handlers::get_server(scheduler))
}

fn post_server(
    scheduler: Scheduler,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let scheduler = warp::any().map(move || scheduler.clone());
    warp::post()
        .and(warp::path!("schedule" / "server"))
        .and(scheduler)
        .and(warp::body::json())
        .and_then(|scheduler, json: HashMap<String, String>| handlers::post_server(scheduler, json))
}
