


use crate::*;

pub fn gen_passwords(){
    #[derive(Clone)]
    pub enum Pswds<'p>{
        Admin(&'p [u8]),
        Dev(&'p [u8]),
    }

    dotenv().expect("⚠️ .env file not found");
    let salt = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
    let salt_bytes = salt.as_bytes();
    
    let dev_pswd = "d3v@%$^$3hjsD";
    let dev_password_bytes = dev_pswd.as_bytes();
    let dev_pass = argon2::hash_encoded(dev_password_bytes, salt_bytes, &argon2::Config::default());


    let admin_pswd = "4dmin@%$^$3hjsD";
    let admin_password_bytes = admin_pswd.as_bytes();
    let admin_pass = argon2::hash_encoded(admin_password_bytes, salt_bytes, &argon2::Config::default());

    info!("dev password is {:?}", dev_pass.clone().unwrap());
    info!("admin password is {:?}", admin_pass.clone().unwrap());

    let pswds = [Pswds::Admin(admin_pass.unwrap().as_bytes()), Pswds::Dev(dev_pass.unwrap().as_bytes())];
}