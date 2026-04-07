/// Example from https://learnxinyminutes.com/texinfo/
use winnow::Parser;
fn main() -> anyhow::Result<()> {
    let input = include_str!("simple-document.info");
    match mr::manual_main_file.parse(input) {
        Ok(f) => {
            dbg!(f);
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }
    Ok(())
}
