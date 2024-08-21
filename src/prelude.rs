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
                internal_utils::lua_printer(&lua_args, printer );
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

    fn invoke_setup<'a>(&'a mut self, arg_list: Vec<(Option<String>, Value)> ) -> Result<Value<'a>> {
        let tbl = self.lua.create_table()?;
        for pair in &arg_list {
            let val = &pair.1;
            if let Some(key) = &pair.0 {
                tbl.set(key.clone(),val)?;
            }else{
                tbl.push(val)?;
            }
        }

        let lua_setup: Function = self.lua.globals().get("setup")?;
        let setup_result = lua_setup.call::<_, Value>(tbl)?;
        Ok( setup_result )
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

    static LOGGER_BUF: Mutex<String> = Mutex::new(String::new());
    #[test]
    fn logger() -> Result<()> {
        let ss = r#"["simple", "2", "nil", "another"]"#;
        {
            let _ilua = LuaInside::new("print('simple', 2, nil, 'another')", 
                |arg_list|{
                    let mut ns = LOGGER_BUF.lock().unwrap();
                    *ns = format!( "{:?}", arg_list );
                } )?;
        }
        assert!( ss == *LOGGER_BUF.lock().unwrap() ); 
        Ok(())
    }

    static SETUP_BUF: Mutex<String> = Mutex::new(String::new());
    #[test]
    fn lua_setup_to_print() -> Result<()> {
        {
            let mut ilua = LuaInside::new("function setup() print('1234') end", 
                |arg_list|{
                    let mut ns = SETUP_BUF.lock().unwrap();
                    *ns = format!( "{:?}", arg_list );
                    println!("ns = <{}>", *ns);
                } )?;
            ilua.invoke_setup(vec![])?;
        }
        assert!( r#"["1234"]"# == *SETUP_BUF.lock().unwrap() ); 
        Ok(())
    }

    #[test]
    fn lua_setup() -> Result<()> {
        let mut ilua = LuaInside::new("function setup(a1) return a1+2 end", |_|{} )?;
        let arg_list = vec![ (None, mlua::Value::Integer(40)) ];
        let res = ilua.invoke_setup(arg_list)?.as_integer();
        assert!( Some::<i64>(42) == res ); 
        Ok(())
    }

    #[test]
    fn lua_setup_ext() -> Result<()> {
        let mut ilua = LuaInside::new("function setup(a1,b2) return b2,a1 end", |_|{} )?;
        let mut res = None;
        {
            let arg_list;
            {
                let tbl = ilua.lua.create_table()?;
                tbl.push( 2 )?;
                tbl.push( "4" )?;
                arg_list = mlua::Value::Table(tbl);
            }
            //res = ilua.invoke_setup(&arg_list)?.as_integer();
        }
        assert!( Some::<i64>(42) == res ); 
        Ok(())
    }
}

