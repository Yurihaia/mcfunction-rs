use std::collections::{HashMap, HashSet};

use mcf_util::Identifier;

#[derive(Debug, Clone)]
pub struct Datapack {
    advancements: HashMap<Identifier, HashSet<Identifier>>,
    loot_tables: HashSet<Identifier>,
    recipes: HashSet<Identifier>,
    structures: HashSet<Identifier>,
    functions: HashMap<Identifier, ()>,
    predicates: HashSet<Identifier>,
    block_tags: HashMap<Identifier, HashSet<Identifier>>,
    entity_tags: HashMap<Identifier, HashSet<Identifier>>,
    item_tags: HashMap<Identifier, HashSet<Identifier>>,
    function_tags: HashMap<Identifier, HashSet<Identifier>>,
    fluid_tags: HashMap<Identifier, HashSet<Identifier>>,
}
