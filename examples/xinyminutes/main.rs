use winnow::{LocatingSlice, Parser as _};

/// Example from https://learnxinyminutes.com/texinfo/
fn main() -> anyhow::Result<()> {
    let input = include_str!("simple-document.info");
    match mr::info::parse::nonsplit_info_file.parse(LocatingSlice::new(input)) {
        Ok(f) => {
            dbg!(f);
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }
    Ok(())
}
