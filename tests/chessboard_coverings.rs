use std::collections::HashMap;
use std::mem;
use xdd::{BDDFactory, DecisionDiagramFactory, NodeIndex, NoMultiplicity, VariableIndex, ZDDFactory};

type Site = [usize;2];
type SiteIndex = usize;
type Tile = Vec<SiteIndex>;
type TileIndex = usize;

#[derive(Default)]
struct TilingProblem {
    sites : Vec<Site>,
    site_index_by_site : HashMap<Site,SiteIndex>,
    tiles : Vec<Tile>,
    /// tiles_covering_a_site[site_index] is a list containing tile_index iff tiles[tile_index] contains site_index.
    tiles_covering_a_site : Vec<Vec<TileIndex>>,
}

impl TilingProblem {
    fn add_site(&mut self,s:Site) -> SiteIndex {
        let index = self.sites.len();
        self.sites.push(s);
        self.site_index_by_site.insert(s,index);
        self.tiles_covering_a_site.push(Vec::new());
        index
    }
    fn add_tile(&mut self,tile:Tile) {
        let index = self.tiles.len();
        for &s in &tile {
            self.tiles_covering_a_site[s].push(index);
            self.tiles_covering_a_site[s].sort();
        }
        self.tiles.push(tile);
    }
    /// If all the sites on the tile exist, add it and return true. Otherwise return false.
    fn add_tile_containing_sites(&mut self,sites:&[Site]) -> bool {
        let mut tile = Vec::new();
        for s in sites {
            if let Some(index) = self.site_index_by_site.get(s) { tile.push(*index); } else { return false; }
        }
        self.add_tile(tile);
        true
    }
    fn find_tiling_solution<F: DecisionDiagramFactory<u32,NoMultiplicity>>(&self) -> (F, NodeIndex<u32,NoMultiplicity>) {
        let mut factory = F::new(self.tiles.len() as u16);
        let mut constraints = Vec::new();
        for tiles_covering_site in &self.tiles_covering_a_site {
            let constraint_for_that_site = factory.exactly_one_of(& tiles_covering_site.iter().map(|t|VariableIndex(*t as u16)).collect::<Vec<_>>());
            constraints.push(constraint_for_that_site);
        }
        constraints.reverse(); // much faster to merge later tiles first.
        let node = factory.poly_and(&constraints).unwrap();
        (factory,node)
    }
}

/// Define a tiling problem as a chessboard with dominoes.
fn setup_chessboard_tiled_with_dominoes(side_length_wanted:usize) -> TilingProblem {
    let mut problem = TilingProblem::default();
    for y in 0..side_length_wanted {
        for x in 0..side_length_wanted {
            problem.add_site([x,y]);
        }
    }
    for y in 0..side_length_wanted {
        for x in 0..side_length_wanted {
            // add tile going to right
            problem.add_tile_containing_sites(&[[x,y],[x+1,y]]);
            // add tile going down.
            problem.add_tile_containing_sites(&[[x,y],[x,y+1]]);
        }
    }
    assert_eq!(side_length_wanted*(side_length_wanted-1)*2,problem.tiles.len());
    problem
}

/// Define a tiling problem as a chessboard with mononimoes, dominoes, and trionimoes.
fn setup_chessboard_tiled_with_up_to_trionimoes(side_length_wanted:usize) -> TilingProblem {
    let mut problem = TilingProblem::default();
    for y in 0..side_length_wanted {
        for x in 0..side_length_wanted {
            problem.add_site([x,y]);
        }
    }
    for y in 0..side_length_wanted {
        for x in 0..side_length_wanted {
            // add mononimo
            problem.add_tile_containing_sites(&[[x,y]]);
            // add tile going to right
            problem.add_tile_containing_sites(&[[x,y],[x+1,y]]);
            // add tile going down.
            problem.add_tile_containing_sites(&[[x,y],[x,y+1]]);
            // add trionimoes
            problem.add_tile_containing_sites(&[[x,y],[x+1,y],[x+2,y]]);
            problem.add_tile_containing_sites(&[[x,y],[x+1,y],[x+1,y+1]]);
            problem.add_tile_containing_sites(&[[x,y],[x+1,y],[x,y+1]]);
            problem.add_tile_containing_sites(&[[x,y],[x+1,y+1],[x,y+1]]);
            problem.add_tile_containing_sites(&[[x,y],[x,y+1],[x,y+2]]);
            problem.add_tile_containing_sites(&[[x+1,y],[x,y+1],[x+1,y+1]]);
        }
    }
    problem
}

/// Count using a decision diagram, given a creator function for the factory taking the number of variables.
fn count_tiling<F: DecisionDiagramFactory<u32, NoMultiplicity>>(problem:TilingProblem) -> u128 {
    let (mut factory ,solution) = problem.find_tiling_solution::<F>();
    let original_len = factory.len();
    let renamer = factory.gc([solution]);
    let solution = renamer.rename(solution).unwrap();
    let solutions : u128 = factory.number_solutions(solution);
    let gc_len = factory.len();
    println!("Original len {} gc len {} solutions {}",original_len,gc_len,solutions);
    solutions
}


#[test]
fn count_dominoes_bdd() {
    let solutions = count_tiling::<BDDFactory<u32,NoMultiplicity>>(setup_chessboard_tiled_with_dominoes(8));
    assert_eq!(solutions,12988816); // See Knuth, "The art of Computer programming Volume 4, Fascicle 1, Binary Decision Diagrams", section 7.1.4, p119
}

#[test]
fn count_dominoes_zdd() {
    let solutions = count_tiling::<ZDDFactory<u32,NoMultiplicity>>(setup_chessboard_tiled_with_dominoes(8));
    assert_eq!(solutions,12988816); // See Knuth, "The art of Computer programming Volume 4, Fascicle 1, Binary Decision Diagrams", section 7.1.4, p119
}


#[test]
fn count_dominoes_dynamic_programming() {
    let mut input_buffer = vec![0u128;256];
    let mut output_buffer = vec![0u128;256];
    input_buffer[0]=1;
    for y in 0..8 {
        for x in 0..8 {
            let mask : usize = 1<<x;
            for i in 0..256 {
                let already_occupied = (i&mask)!=0;
                let count = input_buffer[i];
                if already_occupied {
                    output_buffer[i-mask]+=count;
                } else {
                    // domino downwards
                    output_buffer[i+mask]+=count;
                    // domino right
                    if x<7 && i&(mask<<1) == 0 { output_buffer[i+(mask<<1)]+=count}
                }
                input_buffer[i]=0;
            }
            mem::swap(&mut input_buffer,&mut output_buffer);
        }
    }
    let solutions = input_buffer[0];
    println!("DP tiling solutions : {solutions}");
    assert_eq!(solutions,12988816); // See Knuth, "The art of Computer programming Volume 4, Fascicle 1, Binary Decision Diagrams", section 7.1.4, p119
}


#[test]
fn count_up_to_trionimoes_bdd() {
    let solutions = count_tiling::<BDDFactory<u32,NoMultiplicity>>(setup_chessboard_tiled_with_up_to_trionimoes(8));
    assert_eq!(solutions,92109458286284989468604); // See Knuth, "The art of Computer programming Volume 4, Fascicle 1, Binary Decision Diagrams", section 7.1.4, p120
}

#[test]
fn count_up_to_trionimoes_zdd() {
    let solutions = count_tiling::<ZDDFactory<u32,NoMultiplicity>>(setup_chessboard_tiled_with_up_to_trionimoes(8));
    assert_eq!(solutions,92109458286284989468604); // See Knuth, "The art of Computer programming Volume 4, Fascicle 1, Binary Decision Diagrams", section 7.1.4, p120
}
