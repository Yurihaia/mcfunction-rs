use crate::arena::{Arena, RawId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Commands {
    arena: Arena<Index, Command>,
    root: Index,
}

impl Commands {
    pub fn generate(root: CommandNode) -> Self {
        let (arena, root) = CommandsBuilder::generate(root);
        Commands { arena, root }
    }

    pub fn root(&self) -> &Command {
        &self[self.root_index()]
    }

    pub fn root_index(&self) -> Index {
        self.root
    }
}

impl ops::Index<Index> for Commands {
    type Output = Command;
    fn index(&self, index: Index) -> &Self::Output {
        &self.arena[index]
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Index(RawId);
arena_id!(Index);

impl From<Index> for usize {
    fn from(item: Index) -> usize {
        item.0.into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    name: String,
    children: Vec<Index>,
    executable: bool,
    node_type: CommandNodeType,
}

impl Command {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn children<'a>(
        &'a self,
        commands: &'a Commands,
    ) -> impl Iterator<Item = (Index, &'a Command)> + 'a {
        (&self.children)
            .into_iter()
            .map(move |ind| (*ind, &commands[*ind]))
    }

    pub fn children_indices(&self) -> &[Index] {
        &self.children
    }

    pub fn executable(&self) -> bool {
        self.executable
    }

    pub fn node_type(&self) -> CommandNodeType {
        self.node_type
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum CommandNodeType {
    #[serde(rename = "root")]
    Root,
    #[serde(rename = "literal")]
    Literal,
    #[serde(rename = "argument")]
    Argument {
        #[serde(flatten)]
        parser_type: ParserType,
    },
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(tag = "parser")]
pub enum ParserType {
    #[serde(rename = "minecraft:nbt_tag")]
    NbtTag,
    #[serde(rename = "minecraft:int_range")]
    IntRange,
    #[serde(rename = "minecraft:time")]
    Time,
    #[serde(rename = "minecraft:mob_effect")]
    MobEffect,
    #[serde(rename = "brigadier:integer")]
    Integer {
        #[serde(default)]
        properties: Range<i32>,
    },
    #[serde(rename = "brigadier:bool")]
    Bool,
    #[serde(rename = "minecraft:column_pos")]
    ColumnPos,
    #[serde(rename = "minecraft:objective_criteria")]
    ObjectiveCriteria,
    #[serde(rename = "minecraft:item_predicate")]
    ItemPredicate,
    #[serde(rename = "minecraft:component")]
    Component,
    #[serde(rename = "minecraft:item_slot")]
    ItemSlot,
    #[serde(rename = "minecraft:entity")]
    Entity { properties: EntityProperties },
    #[serde(rename = "minecraft:nbt_compound_tag")]
    NbtCompoundTag,
    #[serde(rename = "brigadier:string")]
    String { properties: StringProperties },
    #[serde(rename = "minecraft:block_pos")]
    BlockPos,
    #[serde(rename = "minecraft:dimension")]
    Dimension,
    #[serde(rename = "minecraft:message")]
    Message,
    #[serde(rename = "minecraft:item_enchantment")]
    ItemEnchantment,
    #[serde(rename = "minecraft:entity_anchor")]
    EntityAnchor,
    #[serde(rename = "minecraft:color")]
    Color,
    #[serde(rename = "minecraft:nbt_path")]
    NbtPath,
    #[serde(rename = "minecraft:block_predicate")]
    BlockPredicate,
    #[serde(rename = "minecraft:particle")]
    Particle,
    #[serde(rename = "minecraft:vec3")]
    Vec3,
    #[serde(rename = "minecraft:resource_location")]
    ResourceLocation,
    #[serde(rename = "minecraft:function")]
    Function,
    #[serde(rename = "minecraft:rotation")]
    Rotation,
    #[serde(rename = "minecraft:score_holder")]
    ScoreHolder { properties: ScoreHolderProperties },
    #[serde(rename = "brigadier:float")]
    Float {
        #[serde(default)]
        properties: Option<Range<f32>>,
    },
    #[serde(rename = "minecraft:operation")]
    Operation,
    #[serde(rename = "minecraft:entity_summon")]
    EntitySummon,
    #[serde(rename = "brigadier:double")]
    Double,
    #[serde(rename = "minecraft:objective")]
    Objective,
    #[serde(rename = "minecraft:block_state")]
    BlockState,
    #[serde(rename = "minecraft:item_stack")]
    ItemStack,
    #[serde(rename = "minecraft:game_profile")]
    GameProfile,
    #[serde(rename = "minecraft:swizzle")]
    Swizzle,
    #[serde(rename = "minecraft:vec2")]
    Vec2,
    #[serde(rename = "minecraft:scoreboard_slot")]
    ScoreboardSlot,
    #[serde(rename = "minecraft:team")]
    Team,
}

impl PartialEq for ParserType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct StringProperties {
    #[serde(rename = "type")]
    pub string_type: StringType,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct EntityProperties {
    pub amount: EntityAmount,
    #[serde(rename = "type")]
    pub entity_type: EntityType,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct ScoreHolderProperties {
    pub amount: EntityAmount,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum StringType {
    #[serde(rename = "phrase")]
    Phrase,
    #[serde(rename = "word")]
    Word,
    #[serde(rename = "greedy")]
    Greedy,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct Range<T> {
    min: Option<T>,
    max: Option<T>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum EntityAmount {
    #[serde(rename = "single")]
    Single,
    #[serde(rename = "multiple")]
    Multiple,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum EntityType {
    #[serde(rename = "players")]
    Players,
    #[serde(rename = "entities")]
    Entities,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandNode {
    #[serde(flatten)]
    node_type: CommandNodeType,
    children: Option<HashMap<String, CommandNode>>,
    redirect: Option<Vec<String>>,
    #[serde(default)]
    executable: bool,
}

#[derive(Debug)]
struct CommandsBuilder {
    arena: Arena<Index, Command>,
}

impl CommandsBuilder {
    fn generate(root: CommandNode) -> (Arena<Index, Command>, Index) {
        let mut tree = HashMap::new();
        let mut out = CommandsBuilder {
            arena: Arena::new(),
        };
        let rootind = out.register_command(String::new(), root, &mut tree);
        let tree = tree.get("").unwrap();
        out.resolve_command(&tree, &tree, rootind);
        (out.arena, rootind)
    }

    fn register_command(
        &mut self,
        name: String,
        node: CommandNode,
        tree: &mut HashMap<String, CommandTree>,
    ) -> Index {
        let ind = self.arena.push(Command {
            name: name.clone(),
            children: Vec::new(),
            executable: node.executable,
            node_type: node.node_type,
        });
        if let Some(map) = node.children {
            let mut inds: Vec<Index> = vec![];
            let mut tree_branch: HashMap<String, CommandTree> = HashMap::new();
            for (name, child) in map {
                inds.push(self.register_command(name, child, &mut tree_branch))
            }
            self.arena[ind].children = inds;
            tree.insert(name, CommandTree(TreeBranch::Children(tree_branch), ind));
            ind
        } else if let Some(v) = node.redirect {
            tree.insert(name, CommandTree(TreeBranch::Redirect(v), ind));
            ind
        } else if !node.executable {
            tree.insert(name, CommandTree(TreeBranch::Root, ind));
            ind
        } else {
            tree.insert(name, CommandTree(TreeBranch::Children(HashMap::new()), ind));
            ind
        }
    }

    fn resolve_command(&mut self, tree: &CommandTree, root: &CommandTree, root_ind: Index) {
        match &tree.0 {
            TreeBranch::Children(map) => {
                for (_, child) in map {
                    self.resolve_command(child, root, root_ind);
                }
            }
            TreeBranch::Root => {
                self.arena[tree.1].children = vec![root_ind];
            }
            TreeBranch::Redirect(path) => {
                let mut top = root;
                let mut stack = path.iter().rev().collect::<Vec<_>>();
                while let Some(v) = stack.pop() {
                    top = match &top.0 {
                        TreeBranch::Root => root,
                        TreeBranch::Children(hs) => hs.get(v).unwrap(),
                        TreeBranch::Redirect(path) => {
                            for x in (0..path.len()).rev() {
                                stack.push(&path[x]);
                            }
                            root
                        }
                    }
                }
                self.arena[tree.1].children = self.arena[top.1].children.clone();
            }
        }
    }
}

#[derive(Debug)]
struct CommandTree(TreeBranch, Index);

#[derive(Debug)]
enum TreeBranch {
    Root,
    Children(HashMap<String, CommandTree>),
    Redirect(Vec<String>),
}
