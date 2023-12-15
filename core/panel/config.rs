


use crate::*;


pub struct Env{
    vars: serde_json::Value,
}

pub trait EnvExt{
    
    type Vars;
    fn set_vars(&mut self);
    fn get_vars(&self) -> Self::Vars;
}

impl EnvExt for Env{

    type Vars = serde_json::Value;
    fn set_vars(&mut self) {
        todo!()
    }

    fn get_vars(&self) -> Self::Vars {
        self.vars.clone()
    }

}