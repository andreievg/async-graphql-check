use actix_web::{guard, web, App, HttpResponse, HttpServer, Result};
use async_graphql::{
    dataloader::Loader,
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject,
};

use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::time::sleep;

#[derive(SimpleObject, Clone, Debug)]
struct User {
    id: i32,
    name: String,
}

struct UserLoader {
    // In a real application, this would likely be a database connection pool
}

impl Loader<i32> for UserLoader {
    type Value = User;
    type Error = Arc<anyhow::Error>; // Using anyhow for simplicity

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
        println!("Loader: loading users with IDs: {:?}", keys);
        // Simulate database access
        sleep(Duration::from_millis(100)).await;

        let mut users = HashMap::new();
        for &id in keys.iter() {
            // Use itertools unique here
            users.insert(
                id,
                User {
                    id,
                    name: format!("User {}", id),
                },
            );
        }
        Ok(users)
    }
}

struct Query;

#[Object]
impl Query {
    async fn user<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        id: i32,
    ) -> Result<Option<User>, async_graphql::Error> {
        let loader = ctx.data_unchecked::<async_graphql::dataloader::DataLoader<UserLoader>>();
        Ok(loader.load_one(id).await?)
    }

    async fn users<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        ids: Vec<i32>,
    ) -> Result<Vec<Option<User>>, async_graphql::Error> {
        let loader = ctx.data_unchecked::<async_graphql::dataloader::DataLoader<UserLoader>>();
        let result_map = loader.load_many(ids.clone()).await?;
        // Ensure the order of results matches the order of input IDs
        let mut results = Vec::new();
        for id in ids {
            results.push(result_map.get(&id).cloned());
        }
        Ok(results)
    }
}

type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

async fn graphql_handler(
    schema: web::Data<AppSchema>,
    req: async_graphql_actix_web::GraphQLRequest,
) -> async_graphql_actix_web::GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql_handler() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql"),
        )))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let user_loader = UserLoader {};
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(async_graphql::dataloader::DataLoader::new(
            user_loader,
            tokio::spawn,
        ))
        .finish();

    println!("GraphiQL IDE: http://localhost:8080/graphiql");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .service(
                web::resource("/graphql")
                    .guard(guard::Post())
                    .to(graphql_handler),
            )
            .service(
                web::resource("/graphiql")
                    .guard(guard::Get())
                    .to(graphiql_handler),
            )
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
