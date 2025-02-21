use crate::models::post::Post;
use crate::schema::posts::dsl::*;
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::result::Error;

use serde::Deserialize;
use snx::StatusCode;
use snx::{request::Request, Context, Json};

use super::{AppError, Result};

#[derive(Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::posts)]
struct PostPayload {
    title: String,
    body: String,
    published: bool,
}

/// Persists a post to the database.
pub fn store(ctx: Context, req: Request) -> Result<(StatusCode, Json<Post>)> {
    let payload = req.json::<PostPayload>()?;
    let result = payload
        .insert_into(posts)
        .get_result::<Post>(&mut ctx.db.get().unwrap())?;

    Ok((StatusCode::Created, Json(result)))
}

/// Retrieves a list of posts from the database.
pub fn index(ctx: Context, _: Request) -> Result<Json<Vec<Post>>> {
    let results = posts
        .select(Post::as_select())
        .load(&mut ctx.db.get().unwrap())?;

    Ok(Json(results))
}

/// Retrieves a post from the database.
pub fn get(ctx: Context, req: Request) -> Result<Json<Post>> {
    let id_param = req.params.get("id").unwrap().parse::<i32>().unwrap();

    let result = posts
        .find(id_param)
        .get_result(&mut ctx.db.get().unwrap())
        .map_err(|e| match e {
            Error::NotFound => AppError::ResourceNotFound,
            _ => AppError::UnknownDatabaseError(e),
        })
        .unwrap();

    Ok(Json(result))
}

/// Updates a post in the database.
pub fn update(ctx: Context, req: Request) -> Result<()> {
    let id_param = req.params.get("id").unwrap().parse::<i32>().unwrap();
    let payload = req.json::<PostPayload>()?;

    let updated_rows = diesel::update(posts::table())
        .filter(id.eq(id_param))
        .set(payload)
        .execute(&mut ctx.db.get().unwrap())?;

    if updated_rows == 0 {
        return Err(AppError::ResourceNotFound);
    }

    Ok(())
}

/// Deletes a post from the database.
pub fn destroy(ctx: Context, req: Request) -> Result<()> {
    let id_param = req.params.get("id").unwrap().parse::<i32>().unwrap();

    let deleted_rows = diesel::delete(posts::table())
        .filter(id.eq(id_param))
        .execute(&mut ctx.db.get().unwrap())?;

    if deleted_rows == 0 {
        return Err(AppError::ResourceNotFound);
    }

    Ok(())
}
