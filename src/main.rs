#![allow(dead_code)]

mod calls;
use std::process::exit;

use calls::{FnDef, process_funcs_in_dir};


fn call_matches(specified_function: &str, full_function_name: &str) -> bool {
    match (specified_function.contains("::"), full_function_name.contains("::")) {
        (true, true) | (false, false) => specified_function == full_function_name,
        (true, false)                 => false,
        (false, true)                 => {
            let index = full_function_name.rfind("::").unwrap();
            specified_function == &full_function_name[index + 2..]
        }
    }
}

fn print_callers(callee: &str, n: usize, prefix: &str, funcs: &Vec<FnDef>) {
    for func in funcs {
        if func.calls.iter().any(|s| s.as_str() == callee) {
            println!("{}{}", prefix, func.name);
            if n != 1 {
                print_callers(&func.name, n-1, &format!("{}{}", prefix, "    "), funcs);
            }
        }
    }
}

fn _print_callees(caller: &str, n: usize, prefix: &str, funcs: &Vec<FnDef>, excludes: &Vec<String>, mut recurs: Vec<String>) {
    if excludes.iter().any(|s| s.as_str() == caller) {
        return;
    }
    if recurs.iter().any(|s| s.as_str() == caller) {
        println!("{}{} -^", prefix, caller);
        return;
    }
    recurs.push(caller.to_string());
    let mut found = false;
    for func in funcs {
        if call_matches(caller, &func.name) {
            found = true;
            println!("{}{}", prefix, &func.name);
            if n != 0 {
                for call in &func.calls {
                    _print_callees(call, n-1, &format!("{}{}", prefix, "|   "), funcs, excludes, recurs.clone()); // 
                }
            }
        }
    }
    if !found {
        println!("{}{}", prefix, caller);
    }
}

fn print_callees(caller: &str, n: usize, funcs: &Vec<FnDef>) {
    _print_callees(caller, n, "", funcs, &get_excludes(), vec![]); // 
}


fn print_functions_structure(funcs: &Vec<FnDef>) {
    let names: Vec<String> = funcs.iter().map(|x| x.name.clone()).collect();
    for func in funcs {
        println!("{}", func.to_string(&names));
    }
}

fn get_excludes() -> Vec<String> {
    vec![
        "?::unwrap",
        "?::expect",
        "?::ok",
        //
        "?::iter",
        "?::any",
        "?::map",
        //
        "?::len",
        "?::collect",
        "?::contains",
        "?::push",
        //
        "?::as_str",
        "?::as_string",
        "?::to_str",
        "?::to_string",
        "?::clone",
        //
        "?::parse",
        //
        "String::new",
        "Vec::new",
        "Box::new",
        "HashMap::new",
    ].iter().map(|s| s.to_string()).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        println!("usage: {} e|r <rust_dir> <func_name> <recurs_level>", &args[0]);
        exit(1);
    }
    let e_r = &args[1]; // e for callees, r for callers
    let dir = &args[2];
    let call_name = &args[3];
    let recurs_level = args[4].parse::<usize>()
        .expect("4th argument must be a number");
    let funcs = process_funcs_in_dir(&format!("{}{}", dir, "/src"));
    //print_functions_structure(&funcs);
    //
    if e_r == "e" {
        print_callees(call_name, recurs_level, &funcs);
    } else if e_r == "r" {
        println!("{}", call_name);
        print_callers(call_name, recurs_level, "    ", &funcs);
    } else {
        panic!("must pass e or r as first argument to {}", &args[0]);
    }
    
    /*
     */
}
