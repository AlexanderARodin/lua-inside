use mlua::{prelude::*,Function,Variadic,Value};
use anyhow::Result;


pub struct LuaInside {
    #[allow(dead_code)]
    lua: Lua,
}

#[allow(dead_code)]
impl LuaInside {
    fn new(lua_code: &str, printer: fn(Vec<String>)->() ) -> Result<Self> {
        // create new mlua instance
        let lua = Lua::new();
        
        {
            // register alter-PRINT
            let lua_print = lua.create_function( move |_, lua_args: Variadic<Value>| {
                lua_printer(&lua_args, printer );
                Ok(())
            })?;
            lua.globals().set("print", lua_print)?;

            // try to Load and Exec
            lua.load( lua_code ).exec()?;
        }
        
        // return new configured LuaInside
        println!("--> [+] LuaInside");
        Ok( Self{ lua: lua } )
    }

    fn invoke_setup<'a>(&'a mut self, args: &Value<'a> ) -> Result<Value<'a>> {
        let lua_setup: Function = self.lua.globals().get("setup")?;
        let setup_result = lua_setup.call::<_, Value>(args)?;
        Ok( setup_result )
    }
}

impl Drop for LuaInside {
    fn drop(&mut self) {
        println!("<-- [-] LuaInside");
    }
}

// // // // // // // //
fn lua_printer(args: &Variadic<Value>, printer: fn(Vec<String>)->() ) {
    let mut arg_list: Vec<String> = Vec::new();
    for item in args.iter() {
        arg_list.push( match item.to_string() {
            Ok(s) => s,
            Err(_) => String::from("<error>"),
        });
    }
    printer( arg_list );
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
        let _ = LuaInside::new("-- g o o d  c o d e", |_|{} )?;
        Ok(())
    }

    #[test]
    fn fail_loading() -> Result<()> {
        match LuaInside::new("b r o k e n  c o d e", |_|{} ) {
            Err(_) => return Ok(()),
            Ok(_) => return Err( anyhow!("Must be a Lua syntax Error") ),
        }
    }

    static SS: &str = r#"["simple", "2", "nil", "another"]"#;
    static NS: Mutex<String> = Mutex::new(String::new());
    #[test]
    fn logger() -> Result<()> {
        {
            let _ilua = LuaInside::new("print('simple', 2, nil, 'another')", 
                |arg_list|{
                    let mut ns = NS.lock().unwrap();
                    *ns = format!( "{:?}", arg_list );
                } )?;
        }
        assert!( SS == *NS.lock().unwrap() ); 
        Ok(())
    }
}

