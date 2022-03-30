use super::db::DbPool;
use super::diesel::prelude::*;
use crate::models::{Session, User};
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
use derive_more::{Display, Error};
use diesel::pg::PgConnection;
use futures::future::{ready, Ready};
use futures_util::future::{FutureExt, LocalBoxFuture};
use serde::Deserialize;
use std::rc::Rc;

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
    auth_data: Option<Rc<AuthData>>,
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
                let session = Session::get_current(&conn, &token);

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
            auth_data: self.auth_data.clone(),
            service: Rc::new(service),
        }))
    }
}

pub struct Authenticated(AuthenticationInfo);

impl Authenticated {
    pub fn get_user(&self, conn: &PgConnection) -> User {
        let session = &self.0.session;
        session.get_user(conn)
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
    form: web::Form<LoginFormData>,
) -> impl Responder {
    use super::schema::users::dsl::*;

    let conn = pool.get().unwrap();
    let user = users
        .filter(username.eq(&form.username))
        .first::<User>(&conn)
        .unwrap(); // TODO need to handle bad username more gracefully

    let valid = verify(&form.password, &user.password).unwrap();
    let redirect = if valid {
        let session = user.new_session(&conn);
        req_id.remember(session.token.clone());
        "/"
    } else {
        "/login/"
    };
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, redirect))
        .finish()
}

#[get("/logoff/")]
async fn logoff(id: Identity) -> impl Responder {
    id.forget();
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, "/login/"))
        .finish()
}

// fn get_user_by_session_token(conn: &PgConnection, token_: &str) -> Option<User> {
// use super::schema::sessions::dsl::{sessions, token};
// use super::schema::users::dsl::{id, users};

// let db_session = sessions
// .filter(token.eq(token_))
// // TODO >= however many hours ago.
// .first::<DbSession>(conn);
// match db_session {
// Ok(s) => {
// let user = users.filter(id.eq(s.user_id)).first::<User>(conn).unwrap();
// Some(user)
// }
// _ => None,
// }
// }
