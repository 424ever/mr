fn main() -> anyhow::Result<()> {
    let input = include_str!("no_nodes.info");
    match mr::info::read_nonsplit_manual(input) {
        Ok(f) => {
            dbg!(f);
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }
    Ok(())
}
