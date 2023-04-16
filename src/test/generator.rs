use std::fs;
use std::fs::File;
use rand::Rng;

pub struct Generator {
    pub(crate) variables: usize,
    pub(crate) clauses: usize,

}

impl Generator {
    pub fn generate(&self, filename: &str) {
        let mut rang = rand::thread_rng();
        let mut commands = vec![
          "(set-option :print-success false)".to_string(),
          "(set-logic QF_UF)".to_string(),
        ];
        (0..self.variables).for_each(|i| {
          commands.push(format!("(declare-fun p{} () Bool)", i))
        });
        for _ in 0..self.clauses {
            let vars: Vec<_> = (0..3).map(|_| {
                let var = format!("p{}", rang.gen_range(0..self.variables));
                if rang.gen::<bool>() {
                    var
                } else {
                    format!("(not {})", var)
                }
            }).collect();
            let clause = format!("(assert (or {} (or {} {})))", vars[0], vars[1], vars[2]);
            commands.push(clause);
        }
        commands.push("(check-sat)".to_string());
        commands.push("(exit)".to_string());
        let total = commands.join("\n");
        let mut output = File::create(filename).unwrap();
        use std::io::Write;
        write!(output, "{}", total).expect("Failed to write to file.");
    }
}

// #[test]
// fn test_gen() {
//     let gen = Generator {
//         variables: 10,
//         clauses: 10,
//     };
//     gen.generate();
// }