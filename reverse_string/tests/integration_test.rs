extern crate  reverse_string;

use reverse_string::reverse;

fn main() {
    let original = "corinthhians";
    let inverted = reverse(original);
    println!("Original: {}", original);
    println!("Inverted: {}", inverted);
}
git add .
git commit -m "Configuração inicial do projeto e .gitignore"