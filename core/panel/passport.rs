


use crate::{*, 
    models::users::{UserData, User, UserRole, JWTClaims}, 
    constants::FETCHED, misc::Response
};




pub trait Passport{

    type Request;
    fn get_user(&self, role: Option<UserRole>, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<JWTClaims, PanelHttpResponse>;

}

impl Passport for HttpRequest{

    /* 
        HttpRequest is not Send because compiler says:
            captured value is not `Send` because `&` references cannot be sent unless their referent is `Sync`
        this is because we're casting the &self into &Self::Request 
        and since User::passport is a future object all the types which 
        are being passed into the method must be Send so we can share them
        between threads which in our case `req` is not Send 
    */
    type Request = HttpRequest;

    fn get_user(&self, role: Option<UserRole>, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<JWTClaims, PanelHttpResponse>{

        /*
            HttpRequest doesn't implement Copy thus dereferencing it
            is now allowed since dereferencing, moves out of the type
            itself and return the owned type which is not possible if 
            there is a pointer of the type exists and that type is not
            Copy, we must either use the borrow form of data or pass its 
            clone to other scopes
        */
        let req = self as &Self::Request;

        /* 
            Copy trait is not implemented for req thus we can't dereference it or 
            move out of it since we can't deref a type if it's behind a pointer, 
            in our case req is behind a pointer and we must clone it to pass 
            it to other scopes
        */
        match User::passport_none_sync(req.clone(), role, connection){
            Ok(token_data) => Ok(token_data),
            Err(resp) => Err(resp)
        }

    }
}