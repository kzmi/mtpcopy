use bindings::windows::Error;

mod wpd;
mod glob;
mod command_dir;
mod command_list;
mod finders;

fn main() -> Result<(), Error> {
    futures::executor::block_on(command_list::command_list())?;
    futures::executor::block_on(command_dir::command_dir())?;
    Ok(())
}
