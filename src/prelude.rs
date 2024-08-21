use mlua::prelude::*;
use mlua::{Variadic,Value};
use anyhow::Result;


pub struct LuaInside {
    #[allow(dead_code)]
    lua: Lua,
}

#[allow(dead_code)]
impl LuaInside {
    fn new(printer: fn(Vec<String>)->() ) -> Result<Self> {
        // create new mlua instance
        let lua = Lua::new();
        
        {
            // register alter-PRINT
            let lua_print = lua.create_function( move |_, lua_args: Variadic<Value>| {
                internal_utils::lua_printer(&lua_args, printer );
                Ok(())
            })?;
            lua.globals().set("print", lua_print)?;
        }
        
        // return new configured LuaInside
        println!("--> [+] LuaInside");
        Ok( Self{ lua: lua } )
    }

    fn exec(&mut self, lua_code: &str) -> Result<()> {
        self.lua.load( lua_code ).exec()?;
        Ok(())
    }
}

impl Drop for LuaInside {
    fn drop(&mut self) {
        println!("<-- [-] LuaInside");
    }
}

// // // // // // // //
mod internal_utils {
    use super::*;

    pub(super) fn lua_printer(args: &Variadic<Value>, printer: fn(Vec<String>)->() ) {
        let mut arg_list: Vec<String> = Vec::new();
        for item in args.iter() {
            arg_list.push( match item.to_string() {
                Ok(s) => s,
                Err(_) => String::from("<error>"),
            });
        }
        printer( arg_list );
    }
}



//  //  //  //  //  //  //  //  //  //  
//  //  //  //  //  //  //  //  //  //  
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::sync::Mutex;

    #[test]
    fn ok_creating() -> Result<()> {
        let mut ilua = LuaInside::new( |_|{} )?;
        ilua.exec("-- g o o d  c o d e")?;
        Ok(())
    }

    #[test]
    fn fail_loading() -> Result<()> {
        let mut ilua = LuaInside::new( |_|{} )?;
        match ilua.exec("b r o k e n  c o d e") {
            Err(_) => return Ok(()),
            Ok(_) => return Err( anyhow!("Must be a Lua syntax Error") ),
        }
    }

    static LOGGER_BUF: Mutex<String> = Mutex::new(String::new());
    #[test]
    fn logger() -> Result<()> {
        let ss = r#"["simple", "2", "nil", "another"]"#;
        {
            let mut ilua = LuaInside::new( 
                |arg_list|{
                    let mut ns = LOGGER_BUF.lock().unwrap();
                    *ns = format!( "{:?}", arg_list );
                } )?;
            ilua.exec("print('simple', 2, nil, 'another')")?;
        }
        assert!( ss == *LOGGER_BUF.lock().unwrap() ); 
        Ok(())
    }
}

