/// Example from https://learnxinyminutes.com/texinfo/
fn main() -> anyhow::Result<()> {
    let input = include_str!("simple-document.info");
    match mr::parse_nonsplit_manual(input) {
        Ok(f) => {
            dbg!(f);
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }
    Ok(())
}
