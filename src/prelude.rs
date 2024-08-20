use mlua::{prelude::*,Variadic,Value};
use anyhow::Result;
use std::cell::RefCell;


pub struct LuaInside {
    lua: Lua,
    printer: RefCell<String>,
}

impl LuaInside {
    #[allow(dead_code)]
    fn new(lua_code: &str) -> Result<Self> {
        let new_one = Self {
            lua: Lua::new(),
            printer: RefCell::new(String::from("oops")),
        };
        let printer_clone = new_one.printer.clone();

        let lua_print = new_one.lua.create_function( move |_, lua_args: Variadic<Value>| {
            invoke_lua_print(&lua_args, printer_clone.clone() );
            Ok(())
        })?;
        new_one.lua.globals().set("print", lua_print)?;

        new_one.lua.load( lua_code ).exec()?;
        println!("--> [+] LuaInside");
        Ok( new_one )
    }
}

impl Drop for LuaInside {
    fn drop(&mut self) {
        println!("<-- [-] LuaInside");
    }
}

// // // // // // // //
fn invoke_lua_print(args: &Variadic<Value>, printer: RefCell<String>) {
    {
        let mut a = printer.borrow_mut();
        *a = "simple".to_string() ;
        println!("TRYYYYYYYYYYY");
    }
    print!("LUA:\t");
    for item in args.iter() {
//        print!("{} - ", item);
    }
    print!("\n");
}



//  //  //  //  //  //  //  //  //  //  
//  //  //  //  //  //  //  //  //  //  
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn ok_creating() -> Result<()> {
        let _ = LuaInside::new("-- g o o d  c o d e")?;
        Ok(())
    }

    #[test]
    fn fail_loading() -> Result<()> {
        match LuaInside::new("b r o k e n  c o d e") {
            Err(_) => return Ok(()),
            Ok(_) => return Err( anyhow!("Must be a Lua syntax Error") ),
        }
    }

    #[test]
    fn printer() -> Result<()> {
        let ilua = LuaInside::new("print('simple')")?;
        let s = ilua.printer.borrow();
        println!("------------ <{}>", s);
        assert!( *s == "simple" ); 
        Ok(())
    }
}

// // // // // // // //

/*

pub fn main_lua_loop(lua: Lua, main_lua_code: &str) -> mlua::Result<()> {
    let globals = lua.globals();

    let lua_print = lua.create_function( |_, lua_args: Variadic<Value>| {
        invoke_lua_print(&lua_args);
        Ok(())
    })?;
    globals.set("print", lua_print)?;

    lua.load( main_lua_code ).exec()?;
    let setup_params = lua.create_table()?;
    let call_lua_setup: Function = globals.get("setup")?;
    let _lua_setup_result = call_lua_setup.call::<_, ()>(setup_params)?;
    
    enter_loop(&lua, &globals)?;

    Ok(())
}

// // // // // // // //
fn enter_loop(_lua: &Lua, globals: &mlua::Table) -> mlua::Result<()> {
    let call_lua_update: Function = globals.get("update")?;

    for time in 1..5 {
        let txt = call_lua_update.call::<_, String>(time)?;
        println!("time = {} : {}", time, txt);
    }

    Ok(())
}

*/
