use std::{fmt::Display, ops::Add, collections::HashSet};

use itertools::Itertools;

use z3::{
    ast::{Ast, Bool, Int},
    Context, Model,
};

#[macro_export]
macro_rules! or {
    ( $ctx:expr , $c:expr ) => {{
        Bool::or(
            $ctx,
            &(0..$c.len()).map(|i| &$c[i]).collect::<Vec<_>>().as_slice(),
        )
    }};
}

#[macro_export]
macro_rules! and {
    ( $ctx:expr , $c:expr ) => {{
        Bool::and(
            $ctx,
            &(0..$c.len()).map(|i| &$c[i]).collect::<Vec<_>>().as_slice(),
        )
    }};
}

pub type Pos = (usize, usize);
pub type Stripe = (usize, (usize, usize));

#[derive(Debug)]
pub struct Akari {
    board: Vec<Vec<char>>,
}

impl Akari {
    /// Get a list of stripes.
    /// 
    /// a strip describes a section in a row on which a bulb can lie. 
    /// 
    /// # Example
    /// 
    /// The following example has three stripes:
    /// - (row: 0, (start: 0, end: 1))
    /// - (row: 0, (start: 3, end: 4))
    /// - (row: 1, (start: 0, end: 4))
    /// 
    /// ```txt
    /// - - 2 - -
    /// - - - - -
    /// ```
    fn get_stripes(&self) -> Vec<Stripe> {
        let mut stripes = vec![];

        for (r, row) in self.board.iter().enumerate() {
            stripes.append(
                &mut row
                    .iter()
                    .enumerate()
                    .fold(vec![vec![]], |mut vec, (c, value)| {
                        if *value == '-' {
                            vec.last_mut().unwrap().push(c);
                        } else {
                            vec.push(vec![]);
                        }
                        vec
                    })
                    .into_iter()
                    .filter(|vec| vec.len() != 0)
                    .map(|vec| (r, (vec[0], *vec.last().unwrap())))
                    .collect(),
            );
        }

        stripes
    }

    fn get_vars<'a>(&self, context: &'a Context) -> Vec<Int<'a>> {
        self.get_stripes()
            .into_iter()
            .map(|(r, (start, end))| {
                Int::new_const(context, format!("r_{}_c_{}-{}", r, start, end))
            })
            .collect()
    }

    pub fn get_asserts<'a>(&self, context: &'a Context) -> Vec<Bool<'a>> {
        let mut asserts: HashSet<Bool> = Default::default();
        let dim = self.get_dim();

        let stripes = self.get_stripes();
        let vars: Vec<Int> = self.get_vars(context);

        // 1.
        // Limit bulbs to the be within the range of the stripe (start..end).
        // bulb = end+1 means there is no bulb on the stripe.
        for (var, (r, (start, end))) in vars.iter().zip(&stripes) {
            asserts.insert(var.ge(&Int::from_u64(context, ((dim.1+1) * r + start) as u64)));
            asserts.insert(var.le(&Int::from_u64(context, ((dim.1+1) * r + end + 1) as u64)));
        }

        // 2.
        // Add the conditions that the cells have exactly as many adjacent bulbs 
        // as necessary.
        for ((r, c), val) in self
            .board
            .iter()
            .enumerate()
            .map(|(r, row)| {
                row.into_iter()
                    .cloned()
                    .enumerate()
                    .map(move |(c, val)| ((r, c), val))
            })
            .flatten()
            .filter(|(_, val)| *val != '-' && *val != 'x')
            .collect::<Vec<(Pos, char)>>()
        {
            let n = val.to_string().parse::<u8>().unwrap();

            // The constraints for the neighbouring stripes with a light bulb on 
            // the neighbouring field.
            let neighbours = get_neighbour_stripes((r, c), &stripes)
                .into_iter()
                .map(|(index, pos)| {
                    vars[index]._eq(&Int::from_u64(context, ((dim.1+1) * pos.0 + pos.1) as u64))
                })
                .collect::<Vec<Bool>>();

            if n == 0 {
                // n = 0 means no neighbouring stripe can have a bulb on the 
                // neighbouring field.
                asserts.insert(or!(context, neighbours).not());
            } else {
                assert!(n <= 4, "Square can only have 0-4 neighbours.");
                for group in unique_permutations(neighbours.clone(), 2) {
                    let others = neighbours
                        .iter()
                        .filter(|c| !group.contains(c))
                        .cloned()
                        .collect::<Vec<Bool>>();

                    // All of a group <-> No
                    // asserts.insert(and!(context, group).iff(&or!(context, others).not()));
                }
            }
        }

        // 3. 
        // Add constraints to prevent bulbs from illuminating each other within 
        // a row.
        for c in 0..2 {
            let mut start_row = 0;
            
            'row: for r in 0..dim.0 {
                if self.board[r][c] == '-' && r != dim.0-1 {
                    continue;
                }

                if r != start_row || r == dim.0-1 {
                    let end_row = match r == dim.0-1 {
                        true => r,
                        false => r-1
                    };

                    let stripes_in_r_and_c = stripes
                        .iter()
                        .enumerate()
                        .filter(|(_, (sr, (start, end)))| *start <= c && *end >= c && start_row <= *sr && end_row >= *sr)
                        .map(|(i, stripe)| (i, *stripe))
                        .collect::<Vec<(usize, Stripe)>>();

                    if stripes_in_r_and_c.len() <= 1 {
                        continue 'row;
                    }

                    // println!("{:?}: {:?}", ((start_row, end_row), c), stripes_in_r_and_c.len());

                    for i in 0..stripes_in_r_and_c.len() {
                        for j in (i+1)..stripes_in_r_and_c.len() {
                            let (a_index, (a_row, _)) = stripes_in_r_and_c[i];
                            let (b_index, (b_row, _)) = stripes_in_r_and_c[j];
                            
                            let a_mod = vars[a_index].modulo(&Int::from_u64(context, (dim.1+1) as u64));
                            let b_mod = vars[b_index].modulo(&Int::from_u64(context, (dim.1+1) as u64));

                            let constr = a_mod._eq(&b_mod).not();

                            if !asserts.contains(&constr) {
                                asserts.insert(a_mod._eq(&b_mod).not());

                                println!("(a_row: {}, b_row: {}){:?}", a_row, b_row, a_mod._eq(&b_mod).not())
                            }
                        }
                    }
                    // dbg!(stripes_in_r_and_c);
                }

                start_row = r+1;
            }
        }

        asserts.into_iter().collect()
    }

    pub fn set_solution<'a>(&mut self, context: &'a Context, model: Model) {
        let dim = self.get_dim();
        let stripes = self.get_stripes();
        let vars: Vec<Int> = self.get_vars(context);

        for (var, (r, (_, end))) in vars.iter().zip(stripes) {
            let c = model.eval(var, false).unwrap().as_u64().unwrap() as usize % (dim.1+1);

            println!("r: {}, c: {}", r, c);

            if c <= end {
                self.board[r][c] = 'o';
            }
        }
    }

    pub fn get_dim(&self) -> (usize, usize) {
        (self.board.len(), self.board[0].len())
    }
}

impl From<String> for Akari {
    fn from(s: String) -> Self {
        let board = s
            .split("\n")
            .filter(|row| *row != "")
            .map(|row| row.chars().filter(|c| *c != ' ').collect::<Vec<char>>())
            .collect::<Vec<Vec<char>>>();

        let dim = (board.len(), board.get(0).unwrap_or(&vec![]).len());

        assert!(
            dim.0 > 0 && dim.1 > 0,
            "Board dimension must be greater than 0."
        );

        for row in board.iter() {
            assert!(row.len() == dim.1, "All rows must have the same size.");
        }

        Self { board }
    }
}

impl Display for Akari {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.board.iter().for_each(|row| {
            row.iter().for_each(|column| {
                f.write_fmt(format_args!("{} ", column));
            });
            f.write_str("\n");
        });

        Ok(())
    }
}

fn unique_permutations<'ctx>(constraints: Vec<Bool<'ctx>>, n: usize) -> Vec<Vec<Bool<'ctx>>> {
    constraints
        .into_iter()
        .permutations(n)
        .map(|mut constraints| {
            constraints.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
            constraints
        })
        .unique()
        .collect::<Vec<_>>()
}

fn get_neighbour_stripes((r, c): (usize, usize), stripes: &Vec<Stripe>) -> Vec<(usize, Pos)> {
    stripes
        .iter()
        .enumerate()
        .filter_map(|(i, (sr, (start, end)))| {
            if r == *sr {
                if c == end + 1 {
                    Some((i, (r, *end)))
                } else if c == start - 1 {
                    Some((i, (r, *start)))
                } else {
                    None
                }
            } else if (r > 0 && r - 1 == *sr) || r + 1 == *sr {
                if c >= *start && c <= *end {
                    Some((i, (*sr, c)))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}
