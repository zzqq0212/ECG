use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    fs::File, 
    result::Result, 
};
use std::io::Write; 

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Syscall {
    name: String,
    arguments: Vec<Argument>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Argument {
    name: String,
    meta_type: String,
    #[serde(rename = "type")]
    arg_type: String,
    size: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Structs {
    name: String,
    fields: Vec<Field>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Field {
    name: String,
    meta_type: String,
    #[serde(rename = "type")]
    field_type: String,
    size: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Root {
    syscall: Vec<Syscall>,
    structs: Vec<Structs>,
}

fn generate_c_syscalls(root: &Root) -> String {
    let mut syscalls_c_code = String::from("const call_t syscalls[] = {\n");

    for syscall in &root.syscall {
        // Dynamically get the arguments count for each syscall
        let args_count = syscall.arguments.len();
        
        // Prepare an arguments representation, assuming {} for simplicity
        let args_representation = "{}"; // Adjust based on your requirements
        
        let syscall_entry = format!(
            "    {{\"{}\", {}, {}, (syscall_t){}}},\n",
            syscall.name, args_count, args_representation, syscall.name
        );
        syscalls_c_code.push_str(&syscall_entry);
    }

    syscalls_c_code.push_str("};\n");
    syscalls_c_code
}
fn generate_fixed_length_c_struct_arrays(root: &Root) -> Vec<String> {
    let mut c_struct_declarations = Vec::new();

    for struct_def in &root.structs {
        let struct_name = &struct_def.name;

        // Format the C array declaration
        let array_declaration = format!("{} {}[5];\n", struct_name, make_c_compatible_name(struct_name));

        c_struct_declarations.push(array_declaration);
    }

    c_struct_declarations
}

// Simplified helper function
fn make_c_compatible_name(name: &str) -> String {
    name.to_lowercase() // Convert the entire string to lowercase
}

fn main() {
    // read josn file from argument
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} -json <config_file>", args[0]);
        return;
    }

    let config_file_path = &args[2];
    let config_file = match fs::read_to_string(config_file_path) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to read config file: {}", e);
            return;
        }
    };
    
    let root: Result<Root, serde_json::Error> = serde_json::from_str(&config_file);
    let rc = match &root {
        Ok(r) => {
            // dbg!(r);
            r.clone()
        },
        Err(e) => {
            println!("Failed to parse JSON: {}", e);
            return;
        }
    };

    let struct_arrays_c_code = generate_fixed_length_c_struct_arrays(&rc);
    // For demonstration, print the generated C code
    for c_code in struct_arrays_c_code {
        println!("{}", c_code);
    }

    let syscalls_c_code = generate_c_syscalls(&rc);
    // For demonstration, print the generated C code
    println!("{}", syscalls_c_code);

    // write this into a file called syscalls.h
    let mut file = match File::create("syscalls.h") {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to create syscalls.h: {}", e);
            return;
        }
    };
    match file.write_all(syscalls_c_code.as_bytes()) {
        Ok(_) => {},
        Err(e) => {
            println!("Failed to write syscalls.h: {}", e);
            return;
        }
    }

    return;
}
