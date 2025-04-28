use lib::{
    compat::function::IntoCallable,
    error::{Error, Result},
    interface,
};

interface!(IoLib {
    println: println.into_callable(),
    print: print.into_callable(),
});

pub fn println(text: String) -> Result<()> {
    println!("{text}");
    Ok(())
}
pub fn print(text: String) -> Result<()> {
    print!("{text}");
    Ok(())
}
