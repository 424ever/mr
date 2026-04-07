use winnow::Parser;

fn main() -> anyhow::Result<()> {
    let input = include_str!("no_nodes.info");
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
