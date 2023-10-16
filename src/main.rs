mod calls;

fn main() {
    let funcs = calls::process_funcs_in_dir(".");
    let names: Vec<String> = funcs.iter().map(|x| x.name.clone()).collect();
    for func in funcs {
        if names.contains(&func.name) {
            println!("{}", func.to_string(&names));
        }
    }
}
