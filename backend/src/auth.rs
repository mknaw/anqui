use std::rc::Rc;

use actix_files::NamedFile;
use actix_identity::{Identity, RequestIdentity};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error, get,
    http::{header, StatusCode},
    middleware::ErrorHandlerResponse,
    post, web, Error, FromRequest, HttpMessage, HttpResponse, Responder,
};
use bcrypt::verify;
use chrono::{Duration, Utc};
use common::models::{Session, User};
use derive_more::{Display, Error};
use diesel::prelude::*;
use futures::future::{ready, Ready};
use futures_util::future::{FutureExt, LocalBoxFuture};
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;

use crate::db::DbPool;

#[derive(Debug, Display, Error)]
#[display(fmt = "auth error")]
pub struct AuthError;

impl error::ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(401).unwrap()
    }
}

pub struct AuthData {
    pub session: Session,
}
pub type AuthenticationInfo = Rc<AuthData>;

pub struct AuthenticateMiddleware<S> {
    _auth_data: Option<Rc<AuthData>>, // TODO do I need this?
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthenticateMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Clone the Rc pointers so we can move them into the async block.
        let srv = self.service.clone();

        async move {
            let token = req.get_identity();
            if let Some(token) = token {
                // See if we can match it to a user.
                let pool = req.app_data::<web::Data<DbPool>>().unwrap();
                let conn = pool.get().unwrap();
                let session = get_current_session(&conn, &token);

                if let Some(session) = session {
                    req.extensions_mut()
                        .insert::<AuthenticationInfo>(Rc::new(AuthData { session }));
                }
            }

            let res = srv.call(req).await?;

            Ok(res)
        }
        .boxed_local()
    }
}

pub struct AuthenticateMiddlewareFactory {
    auth_data: Option<Rc<AuthData>>,
}

impl AuthenticateMiddlewareFactory {
    pub fn new() -> Self {
        AuthenticateMiddlewareFactory { auth_data: None }
    }
}

impl Default for AuthenticateMiddlewareFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthenticateMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticateMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticateMiddleware {
            _auth_data: self.auth_data.clone(),
            service: Rc::new(service),
        }))
    }
}

pub struct Authenticated(AuthenticationInfo);

impl Authenticated {
    // TODO `user` should maybe just live on AuthenticationInfo
    pub fn get_user(&self, conn: &PgConnection) -> User {
        use common::schema::users::dsl::*;
        let session = &self.0.session;
        users
            .filter(id.eq(session.user_id))
            .first::<User>(conn)
            .unwrap()
    }
}

impl FromRequest for Authenticated {
    type Error = AuthError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<AuthenticationInfo>().cloned();
        let result = match value {
            Some(v) => Ok(Authenticated(v)),
            None => Err(AuthError {}),
        };
        ready(result)
    }
}

impl std::ops::Deref for Authenticated {
    type Target = AuthenticationInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn redirect_on_autherror<B, E>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>, E> {
    // Feed me to ErrorHandlers to redirect to /login/ on 401.
    let redirect = HttpResponse::SeeOther()
        .insert_header((header::LOCATION, "/login/"))
        .finish();
    let redirect = ServiceResponse::new(res.request().clone(), redirect);
    Ok(ErrorHandlerResponse::Response(
        redirect.map_into_right_body(),
    ))
}

#[derive(Deserialize)]
pub struct LoginFormData {
    username: String,
    password: String,
}

#[get("/login/")]
async fn login_get() -> impl Responder {
    // Kinda nasty, has to do the same thing as `index` but not extract `Authenticated`. (TODO ?)
    NamedFile::open_async("./frontend/dist/index.html").await
}

#[post("/login/")]
async fn login(
    req_id: Identity,
    pool: web::Data<DbPool>,
    form: web::Json<LoginFormData>,
) -> impl Responder {
    use common::schema::users::dsl::*;

    let conn = pool.get().unwrap();
    let user = users
        .filter(username.eq(&form.username))
        .first::<User>(&conn);

    let err_message;
    if let Ok(user) = user {
        if verify(&form.password, &user.password).unwrap() {
            let session = new_session(&user, &conn);
            req_id.remember(session.token);
            return HttpResponse::Ok().finish();
        } else {
            err_message = "Mot de passe invalide".to_string();
        }
    } else {
        err_message = "Nom d'utilisateur invalide".to_string();
    }
    req_id.forget();
    HttpResponse::Forbidden().body(err_message)
}

#[get("/logout/")]
async fn logout(id: Identity) -> impl Responder {
    id.forget();
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, "/login/"))
        .finish()
}

fn get_current_session(conn: &PgConnection, try_token: &str) -> Option<Session> {
    use common::schema::sessions::dsl::*;

    // TODO prolly could have this in config / env
    let min_ts = Utc::now().naive_utc() - Duration::hours(36);
    sessions
        .filter(token.eq(try_token))
        .filter(created.gt(min_ts))
        .first::<Session>(conn)
        .ok()
}

// TODO probably should return a Result
fn new_session(user: &User, conn: &PgConnection) -> Session {
    use common::schema::sessions::dsl::*;

    diesel::delete(sessions.filter(user_id.eq(user.id)))
        .execute(conn)
        .unwrap();

    let tok: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(25)
        .map(char::from)
        .collect();

    diesel::insert_into(sessions)
        .values((
            user_id.eq(user.id),
            token.eq(tok),
            created.eq(Utc::now().naive_utc()),
        ))
        .get_result(conn)
        .unwrap()
}
