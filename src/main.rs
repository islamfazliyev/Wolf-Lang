use std::{fs, io::{self, BufRead, Read}};
use clap::{Arg, Parser as otherParser};
use std::time::Instant;

use wolflang::{WolfEngine, lexer::lexer, parser};



#[derive(otherParser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    file: Option<String>,
}

fn main() {
    let args = Args::parse();
    if let Some(file_path) = args.file {
        run_file(&file_path);
    } else {
        run();
    }   
    
}

fn run_file(path: &str) {
    
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Dosya okunamadı: {}", e);
            return;
        }
    };
    
    let mut Engine = WolfEngine::new();

    if let Err(e) = Engine.run(&content) {
        eprintln!("{}", e);
    }

}

fn run() {
    println!(r#"                                        
               ##    #                  
              #### ###                  
             ##  ##  #                  
             ##   ###                   
            ###  ########               
           #################            
           ###################          
        #################  ###%         
         ##########################     
        #############################   
      ##%###########################    
        ###########################     
      ######################            
     #####################              
      ###################               
        ################                
          #############                 
             #########                  
               ######                   
                 ###                    
                                        
"#);
    println!("   W O L F L A N G  v0.1");
    println!("---------------------------");
    let mut lines: Vec<String> = Vec::new();
    let mut content;
    loop {
        let stdin = io::stdin();
        let mut buffer: String = String::new();
        for line in stdin.lock().lines() {
            let line = line.expect("read error");

            if line.trim() == "run()" {
                // for döngüsünü bitir
                break;
            }

            lines.push(line);
        }

        content = lines.join("");
        break;
    }
    let tokens = match lexer(&content) {
        Ok(tokens) => {
            tokens
        }
        Err(err) => {
            eprintln!("Lexer error: {}", err);
            return;
        }
    };
    println!("{:?}", tokens);

    let mut parser = parser::Parser::new(tokens);

    while parser.current_token().is_some() {
        if let Err(e) = parser.parse_statement() {
            eprintln!("❌ Parser Error: {:?}", e);
            break;
        }
    }
}


#[cfg(test)]
mod tests {
    use wolflang::WolfEngine;

    #[test]
    fn counter() {
        let mut Engine = WolfEngine::new();
        let mut counter: i64 = 0;
        while counter < 10 {
            
            Engine.push_int("counter", counter);
    
            let code = r#"
                counter = counter + 1
            "#;
    
            if let Err(e) = Engine.run(code) {
                eprintln!("{}", e);
            }
    
            let value = Engine.get_int("counter").unwrap();
            counter = value;
            println!("{:?}", value);
        }
    }
}