use axum::{extract::State, routing::post, Router};
use backend::application::{
    ports::usecase::UsecaseAPI,
    routes::authorized::AuthorizedRoute,
    usecases::{
        hq::{
            commands::{
                cp_model::CPModelSvc, delete_command::DeleteCommand, list_command::ListCommand,
            },
            controls::{
                launch_compute::LaunchCompute, list_compute::ListCompute,
                manage_compute::ManageCompute,
            },
            models::{list::GetListSvc, preview::PreviewSvc},
        },
        template::{
            delete::TemplateDeleteSvc, list::TemplateListSvc, read::TemplateReadSvc,
            write::TemplateWriteSvc,
        },
    },
};

use crate::http::app::{reply, AppState, Ctx, Payload, Reply};

pub fn router() -> Router<AppState> {
    use AuthorizedRoute as R;
    Router::new()
        .route(&R::ListModels.to_string(), post(list_models))
        .route(&R::ModelPreview.to_string(), post(model_preview))
        .route(&R::AgentModelCP.to_string(), post(agent_model_cp))
        .route(&R::CommandList.to_string(), post(command_list))
        .route(&R::CommandDelete.to_string(), post(command_delete))
        .route(&R::TemplateList.to_string(), post(template_list))
        .route(&R::TemplateWrite.to_string(), post(template_write))
        .route(&R::TemplateDelete.to_string(), post(template_delete))
        .route(&R::TemplateRead.to_string(), post(template_read))
        .route(&R::HQInstanceLaunch.to_string(), post(instance_launch))
        .route(&R::HQInstanceControl.to_string(), post(instance_control))
        .route(&R::HQInstanceList.to_string(), post(instance_list))
}

async fn list_models(State(app): State<AppState>, Ctx(ctx): Ctx, Payload(p): Payload) -> Reply {
    let clients = app.clients.start(&ctx);
    reply(GetListSvc::new(&clients, p.try_into()?).exec().await)
}

async fn model_preview(State(app): State<AppState>, Ctx(ctx): Ctx, Payload(p): Payload) -> Reply {
    let clients = app.clients.start(&ctx);
    reply(PreviewSvc::new(&clients, p.try_into()?).exec().await)
}

async fn agent_model_cp(State(app): State<AppState>, Payload(p): Payload) -> Reply {
    reply(
        CPModelSvc::new(app.repos.as_ref(), p.try_into()?)
            .exec()
            .await,
    )
}

async fn command_list(State(app): State<AppState>, Payload(p): Payload) -> Reply {
    reply(
        ListCommand::new(app.repos.as_ref(), p.try_into()?)
            .exec()
            .await,
    )
}

async fn command_delete(State(app): State<AppState>, Payload(p): Payload) -> Reply {
    reply(
        DeleteCommand::new(app.repos.as_ref(), p.try_into()?)
            .exec()
            .await,
    )
}

async fn template_list(State(app): State<AppState>, Ctx(ctx): Ctx) -> Reply {
    reply(TemplateListSvc::new(app.repos.as_ref(), &ctx).exec().await)
}

async fn template_write(State(app): State<AppState>, Ctx(ctx): Ctx, Payload(p): Payload) -> Reply {
    reply(
        TemplateWriteSvc::new(app.repos.as_ref(), &ctx, p.try_into()?)
            .exec()
            .await,
    )
}

async fn template_delete(State(app): State<AppState>, Ctx(ctx): Ctx, Payload(p): Payload) -> Reply {
    reply(
        TemplateDeleteSvc::new(app.repos.as_ref(), &ctx, p.try_into()?)
            .exec()
            .await,
    )
}

async fn template_read(State(app): State<AppState>, Ctx(ctx): Ctx, Payload(p): Payload) -> Reply {
    reply(
        TemplateReadSvc::new(app.repos.as_ref(), &ctx, p.try_into()?)
            .exec()
            .await,
    )
}

async fn instance_launch(State(app): State<AppState>, Ctx(ctx): Ctx, Payload(p): Payload) -> Reply {
    let clients = app.clients.start(&ctx);
    reply(
        LaunchCompute::new(&clients, app.repos.as_ref(), &ctx, p.try_into()?)
            .exec()
            .await,
    )
}

async fn instance_control(
    State(app): State<AppState>,
    Ctx(ctx): Ctx,
    Payload(p): Payload,
) -> Reply {
    let clients = app.clients.start(&ctx);
    reply(ManageCompute::new(&clients, p.try_into()?).exec().await)
}

async fn instance_list(State(app): State<AppState>, Ctx(ctx): Ctx, Payload(p): Payload) -> Reply {
    let clients = app.clients.start(&ctx);
    reply(ListCompute::new(&clients, p.try_into()?).exec().await)
}
