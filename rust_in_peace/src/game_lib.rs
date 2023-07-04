//! This is the game library module.
//! It contains critical functions like get_input(), update_state(), and update_screen()
//! that are crucual for running the game
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::read_to_string;
use std::io::stdout;
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

// Indices of all the objects in the game
pub const LOC_FOREST: usize = 0;
pub const LOC_DUNGEONS: usize = 1;
pub const LOC_CAVE: usize = 2;
pub const LOC_TAVERN: usize = 3;
pub const LOC_VILLAGE: usize = 4;
pub const LOC_STRONGHOLD: usize = 5;
pub const LOC_PLAYER: usize = 6;
pub const LOC_BEAR: usize = 7;
pub const LOC_TROLL: usize = 8;
pub const LOC_BANDITS: usize = 9;

///Distance enum containing all the distance prompts
#[derive(PartialOrd, Ord, PartialEq, Eq, Debug)]
pub enum Distance {
    Player,
    Held,
    Location,
    Here,
    OverThere,
    NotHere,
    Unknown,
}

/// Command enum containing all the command prompts
pub enum Command {
    Drop(String),
    Get(String),
    Attack(String),
    Look(String),
    Go(String),
    Unknown(String),
    Inventory,
    Quit,
    Help,
    Map,
}

/// Get input from the user
impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Drop(_) => write!(f, "drop"),
            Command::Get(_) => write!(f, "get"),
            Command::Attack(_) => write!(f, "attack"),
            Command::Go(_) => write!(f, "go"),
            Command::Inventory => write!(f, "inventory"),
            Command::Look(_) => write!(f, "look"),
            Command::Quit => write!(f, "quit"),
            Command::Unknown(_) => write!(f, "unknown"),
            Command::Help => write!(f, "help"),
            Command::Map => write!(f, "map"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) enum Location {
    Forest,
    Dungeons,
    Cave,
    Tavern,
    Village,
    StrongHold,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Forest => "Look out for tree people.",
            Self::Dungeons => "Be aware of trolls.",
            Self::Cave => "Watch out for bats and look out for light.",
            Self::Tavern => "The tavern is empty. But the fire is still burning in the fireplace.",
            Self::Village => "An abandoned village. It has been ransacked by a group of bandits.",
            Self::StrongHold => "A stronghold. It is heavily guarded by a group of bandits.",
        };

        text.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Consumable {
    name: String,
    description: String,
    health_points: usize,
    location: Location,
}

impl Consumable {
    fn new<T: Into<String>>(
        name: T,
        description: T,
        health_points: usize,
        location: Location,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            health_points,
            location,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Weapon {
    name: String,
    description: String,
    location: Location,
    attack_points: u64,
}

impl Weapon {
    fn new<T: Into<String>>(
        name: T,
        description: T,
        location: Location,
        attack_points: u64,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            location,
            attack_points,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Player {
    name: String,
    location: Location,
    health: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Enemy {
    name: String,
    description: String,
    health: u64,
    attack: u64,
    location: Location,
}

impl Enemy {
    fn new<T: Into<String>>(name: T, description: T, attack: u64, location: Location) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            attack,
            location,
            health: 100,
        }
    }
}

impl Player {
    fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            location: Location::Forest,
            health: 100,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
/// The object struct
pub enum Object {
    Player(Player),
    Weapon(Weapon),
    Consumable(Consumable),
    Enemy(Enemy),
    Location(Location),
}

impl From<Location> for Object {
    fn from(location: Location) -> Self {
        Self::Location(location)
    }
}

impl From<Player> for Object {
    fn from(player: Player) -> Self {
        Self::Player(player)
    }
}

impl From<Weapon> for Object {
    fn from(weapon: Weapon) -> Self {
        Self::Weapon(weapon)
    }
}

impl From<Consumable> for Object {
    fn from(consumable: Consumable) -> Self {
        Self::Consumable(consumable)
    }
}

impl From<Enemy> for Object {
    fn from(enemy: Enemy) -> Self {
        Self::Enemy(enemy)
    }
}

/// Handles any ambiguous directions
#[derive(PartialOrd, Ord, PartialEq, Eq, Debug)]
pub enum AmbiguousOption<T> {
    None,
    Some(T),
    Ambiguous,
}

#[derive(Serialize, Deserialize, Debug)]
/// The world struct
pub struct World {
    pub objects: Vec<Object>,
}

impl TryFrom<Object> for Player {
    type Error = &'static str;

    fn try_from(object: Object) -> Result<Self, Self::Error> {
        match object {
            Object::Player(player) => Ok(player),
            _ => Err("This is not a player."),
        }
    }
}

impl TryFrom<Object> for Weapon {
    type Error = &'static str;

    fn try_from(object: Object) -> Result<Self, Self::Error> {
        match object {
            Object::Weapon(weapon) => Ok(weapon),
            _ => Err("This is not a weapon."),
        }
    }
}

impl TryFrom<Object> for Consumable {
    type Error = &'static str;

    fn try_from(object: Object) -> Result<Self, Self::Error> {
        match object {
            Object::Consumable(consumable) => Ok(consumable),
            _ => Err("This is not a consumable."),
        }
    }
}

impl TryFrom<Object> for Enemy {
    type Error = &'static str;

    fn try_from(object: Object) -> Result<Self, Self::Error> {
        match object {
            Object::Enemy(enemy) => Ok(enemy),
            _ => Err("This is not an enemy."),
        }
    }
}

/// The game struct
impl World {
    pub fn new() -> Self {
        World {
            objects: vec![
                Location::Forest.into(),
                Location::Dungeons.into(),
                Location::Cave.into(),
                Location::Tavern.into(),
                Location::Village.into(),
                Location::StrongHold.into(),
                Player::new("Master of None").into(),
                Enemy::new("Bear", "A bear", 20, Location::Cave).into(),
                Enemy::new("Troll", "A troll", 20, Location::Dungeons).into(),
                Enemy::new("Bandits", "A group of bandits", 30, Location::StrongHold).into(),
                Weapon::new("Sword", "A rusty sword", Location::Dungeons, 20).into(),
                Weapon::new("Bow", "A bow", Location::Tavern, 10).into(),
                Weapon::new("Bones", "Bones of an animal", Location::Cave, 5).into(),
                Weapon::new("Spear", "A spear", Location::Village, 25).into(),
                Consumable::new("Apple", "An apple", 10, Location::Tavern).into(),
                Consumable::new("Potion", "A vial of healing potion (Get it to increase health)  (Hint: Type <get potion> to consume it)", 20, Location::Village).into(),
                // TODO: Model the map (directions).
                // Object {
                //     label: vec!["North".to_string()],
                //     description: "A path to the north leading out of the forest leading to an old Tavern"
                //         .to_string(),
                //     location: Some(LOC_FOREST),
                //     destination: Some(LOC_TAVERN),
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["South".to_string()],
                //     description: "A path to the south leading back to the forest".to_string(),
                //     location: Some(LOC_TAVERN),
                //     destination: Some(LOC_FOREST),
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["East".to_string()],
                //     description: "A path to the east leading to the Dungeons".to_string(),
                //     location: Some(LOC_TAVERN),
                //     destination: Some(LOC_DUNGEONS),
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["West".to_string()],
                //     description: "A path to the west leading to an abandoned village".to_string(),
                //     location: Some(LOC_TAVERN),
                //     destination: Some(LOC_VILLAGE),
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["East".to_string()],
                //     description: "A path to the east leading to the tavern".to_string(),
                //     location: Some(LOC_VILLAGE),
                //     destination: Some(LOC_TAVERN),
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["North".to_string()],
                //     description: "A path to the north leading to a stronghold".to_string(),
                //     location: Some(LOC_VILLAGE),
                //     destination: Some(LOC_STRONGHOLD),
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["South".to_string()],
                //     description: "A path to the south leading to the village".to_string(),
                //     location: Some(LOC_STRONGHOLD),
                //     destination: Some(LOC_VILLAGE),
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["West".to_string()],
                //     description: "A path to the west leading to the Tavern".to_string(),
                //     location: Some(LOC_DUNGEONS),
                //     destination: Some(LOC_TAVERN),
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["North".to_string()],
                //     description: "A path to the north into a cave".to_string(),
                //     location: Some(LOC_DUNGEONS),
                //     destination: Some(LOC_CAVE),
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["South".to_string()],
                //     description: "A path to the south into the dungeons".to_string(),
                //     location: Some(LOC_CAVE),
                //     destination: Some(LOC_DUNGEONS),
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["West".to_string(), "East".to_string(), "South".to_string()],
                //     description: "You see nothing but trees. There is no other path in that direction."
                //         .to_string(),
                //     location: Some(LOC_FOREST),
                //     destination: None,
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["West".to_string(), "East".to_string(), "North".to_string()],
                //     description: "There is no other path in that direction."
                //         .to_string(),
                //     location: Some(LOC_STRONGHOLD),
                //     destination: None,
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["North".to_string(), "".to_string()],
                //     description: "There is no other path in that direction.".to_string(),
                //     location: Some(LOC_TAVERN),
                //     destination: None,
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["East".to_string(),"West".to_string()],
                //     description: "There is no other path in that direction.".to_string(),
                //     location: Some(LOC_VILLAGE),
                //     destination: None,
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["East".to_string(), "South".to_string()],
                //     description:
                //         "You see only big rocks and boulders. There is no other path in that direction."
                //             .to_string(),
                //     location: Some(LOC_DUNGEONS),
                //     destination: None,
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
                // Object {
                //     label: vec!["East".to_string(), "North".to_string(), "West".to_string()],
                //     description: "The cave has no paths in that direction".to_string(),
                //     location: Some(LOC_CAVE),
                //     destination: None,
                //     item: false,
                //     enemy: false,
                //     health: None,
                //     attack: None,
                //     consumable: false,
                // },
            ],
        }
    }

    // We are adding reading from file, first step is to read from file.
    pub fn read_from_file(game_file: &str) -> Result<World, std::io::Error> {
        let game_file_path = Path::new(game_file);
        let game_file_data_res = read_to_string(game_file_path);

        match game_file_data_res {
            Ok(game_file_data) => {
                let deserialized_data: Result<World, ron::error::SpannedError> =
                    ron::from_str(&game_file_data);

                match deserialized_data {
                    Ok(deserialized_ron) => Ok(deserialized_ron),
                    Err(de_err_str) => Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        de_err_str.to_string(),
                    )),
                }
            }
            Err(file_err) => Err(file_err),
        }
    }

    /// Check of the game is over
    pub fn game_over(&self) -> bool {
        // TODO: Return an enum to indicate the kind of game over (won, lost because (list of enemies) remaining, ...).
        let player_health = Player::try_from(self.objects[LOC_PLAYER])
            .map(|player| player.health)
            .unwrap();
        let all_enemies_dead = [LOC_BEAR, LOC_TROLL, LOC_BANDITS]
            .into_iter()
            .filter_map(|index| {
                Enemy::try_from(self.objects[index])
                    .map(|enemy| enemy.health)
                    .ok()
            })
            .all(|health| health == 0);

        if player_health == 0 {
            true
        } else if all_enemies_dead {
            println!("You have defeated all enemies! You win!");
            true
        } else {
            false
        }
    }

    /// Function for getting the type writer effect
    pub fn type_writer_effect(&self, text: &str) {
        for c in text.chars() {
            print!("{}", c);
            stdout().flush().expect("Could not flush stdout");
            thread::sleep(Duration::from_millis(25));
        }
    }

    /// Check if the object has a label
    fn object_with_label(&self, object: &Object, noun: &str) -> bool {
        let object_name = match object {
            Object::Player(player) => player.name,
            Object::Weapon(weapon) => weapon.name,
            Object::Consumable(consumable) => consumable.name,
            Object::Enemy(enemy) => enemy.name,
            Object::Location(location) => format!("{:?}", location),
        };

        object_name.to_lowercase() == noun.to_lowercase()
    }

    /// Get the index of the object
    pub fn object_index(
        &self,
        noun: &str,
        from: Option<usize>,
        max_distance: Distance,
    ) -> AmbiguousOption<usize> {
        let mut result: AmbiguousOption<usize> = AmbiguousOption::None;
        for (position, object) in self.objects.iter().enumerate() {
            if self.object_with_label(object, noun)
                && self.get_distance(from, Some(position)) <= max_distance
            {
                if result == AmbiguousOption::None {
                    result = AmbiguousOption::Some(position);
                } else {
                    result = AmbiguousOption::Ambiguous;
                }
            }
        }
        result
    }

    /// Checks if the object is visible
    fn object_visible(&self, noun: &String) -> (String, Option<usize>) {
        let obj_over_there = self.object_index(noun, Some(LOC_PLAYER), Distance::OverThere);
        let obj_not_here = self.object_index(noun, Some(LOC_PLAYER), Distance::NotHere);

        match (obj_over_there, obj_not_here) {
            // Return none if not a valid command
            (AmbiguousOption::None, AmbiguousOption::None) => {
                ("Invalid command!!".to_string(), None)
            }
            (AmbiguousOption::None, AmbiguousOption::Some(_)) => {
                (format!("You don't see any '{}' here.\n", noun), None)
            }
            // Ambiguous object name
            (AmbiguousOption::Ambiguous, _)
            | (AmbiguousOption::None, AmbiguousOption::Ambiguous) => (
                format!("Please be more specific about which {} you mean. Try typing out the location.\n", noun),
                None,
            ),
            (AmbiguousOption::Some(index), _) => (String::new(), Some(index)),
        }
    }

    /// Lists all objects in a location
    fn list_objects(&self, location: usize) -> (String, u64) {
        let mut result = String::new();
        let mut count: u64 = 0;

        result.push_str("\nYou see:\n");

        for (pos, object) in self.objects.iter().enumerate() {
            let description = match object {
                Object::Weapon(weapon) => weapon.description,
                Object::Consumable(consumable) => consumable.description,
                Object::Enemy(enemy) => enemy.description,
                _ => continue,
            };

            if self.is_containing(Some(location), Some(pos)) {
                count += 1;
                result.push_str(&description);
                result.push('\n');
            }
        }

        (result, count)
    }

    /// Updates state of the game
    pub fn update_state(&mut self, command: &Command) -> String {
        match command {
            Command::Look(noun) => self.do_look(noun),
            Command::Go(noun) => self.do_go(noun),
            Command::Quit => "Quitting.\nThank you for playing!".to_string(),
            Command::Attack(noun) => self.do_attack(noun),
            Command::Drop(noun) => self.do_drop(noun),
            Command::Get(noun) => self.do_get(noun),
            Command::Inventory => self.do_inventory(),
            Command::Help => self.display_help(),
            Command::Map => self.display_locations(),
            Command::Unknown(_) => {
                let invalid_msg = String::from("Invalid command!!\n");
                let help = self.display_help();
                invalid_msg + help.as_str()
            }
        }
    }

    /// Function to perform the attack while attacking an enemy
    pub fn do_use(&mut self, msg: &str, mut obj_health: u64, obj_index: usize) -> u64 {
        let mut split_input = msg.split_whitespace();
        let noun = split_input.nth(1).unwrap_or_default().to_string();
        let (output, obj_opt) = self.object_visible(&noun);

        let object = match obj_opt {
            Some(index) => self.objects[index],
            None => {
                self.type_writer_effect(&output);
                return obj_health;
            }
        };

        let weapon = match object {
            Object::Weapon(w) => w,
            _ => {
                self.type_writer_effect("That is not a weapon!!");
                println!("\nHint: Use the following commands: use <weapon name> or run");
                return obj_health;
            }
        };

        let attack_pwr = weapon.attack_points;
        let mut enemy = match self.objects[obj_index] {
            Object::Enemy(e) => e,
            _ => return obj_health,
        };
        obj_health -= attack_pwr;
        self.type_writer_effect(&format!(
            "You attacked the {}.\nEnemy health: {}",
            enemy.name, obj_health
        ));
        if obj_health == 0 {
            enemy.health = 0;
            return obj_health;
        }
        self.type_writer_effect(&format!("\n\nThe {} attacks", enemy.name));

        // random attack
        let mut rng = rand::thread_rng();
        let attack: u64 = rng.gen_range(0..enemy.attack);

        if attack == 0 {
            self.type_writer_effect("\nYou dodged the attack");
        } else {
            self.type_writer_effect("\nYou got hit");
            let player: Result<Player, _> = self.objects[LOC_PLAYER].try_into();
            let player_health = player
                .map(|mut player| {
                    player.health -= attack;
                    player.health
                })
                .unwrap_or_default();
            self.type_writer_effect(&format!("\nYour health: {}", player_health));
        }

        obj_health
    }

    /// Function to attack an enemy
    pub fn do_attack(&mut self, noun: &String) -> String {
        let (output, obj_opt) = self.object_visible(noun);

        let obj_index = match obj_opt {
            Some(i) => i,
            None => return output,
        };

        let enemy = match self.objects[obj_index] {
            Object::Enemy(e) => e,
            _ => return format!("You can't attack the {}.\n", noun),
        };

        let mut obj_health: u64 = enemy.health;

        if obj_health == 0 {
            return format!("The {} is already dead.\n", enemy.name);
        }
        self.type_writer_effect(&format!("\nYou are attacking the {}.\n", enemy.name));

        println!("\nHint: Use the following commands when attacking: 'use <weapon name>' or 'inventory' or 'run'");

        let player: Player = self.objects[LOC_PLAYER].try_into().unwrap();

        loop {
            if player.health == 0 {
                return "\nYou died".to_string();
            }
            print!("\n> ");
            io::stdout().flush().unwrap();

            let mut command = String::new();
            io::stdin()
                .read_line(&mut command)
                .expect("Failed to read input");
            if command.contains("run") {
                break;
            } else if command.contains("inventory") {
                let list_objects = self.do_inventory();
                self.type_writer_effect(&list_objects);
                continue;
            } else if command.contains("use") {
                obj_health = self.do_use(&command, obj_health, obj_index);
                if obj_health == 0 {
                    break;
                }
            } else {
                println!("\nHint: Use the following commands when attacking: 'use <weapon name>' or 'inventory' or 'run'");
            }
        }
        if obj_health == 0 {
            format!("\nYou killed the {}.\n", enemy.name)
        } else {
            format!(
                "You ran away from the {}.\n",
                enemy.name
            )
        }
    }

    /// Look around the surroundings of the location the player is in
    pub fn do_look(&self, noun: &str) -> String {
        match noun {
            "" => {
                let (list, _) = self.list_objects(self.objects[LOC_PLAYER].location.unwrap());
                format!(
                    " You are in the {}\n {}.\n",
                    self.objects[self.objects[LOC_PLAYER].location.unwrap()].label[0],
                    self.objects[self.objects[LOC_PLAYER].location.unwrap()].description
                ) + list.as_str()
            }
            _ => "Invalid command!!\n".to_string(),
        }
    }

    /// Player goes to the specified location
    pub fn do_go(&mut self, noun: &String) -> String {
        let (output, obj_opt) = self.object_visible(noun);

        match self.get_distance(Some(LOC_PLAYER), obj_opt) {
            Distance::OverThere => {
                self.objects[LOC_PLAYER].location = obj_opt;
                "OK.\n".to_string() + &self.do_look("")
            }
            Distance::NotHere => {
                format!("You don't see any '{}' here.\n", noun)
            }
            Distance::Unknown => output,
            _ => {
                let obj_dist = obj_opt.and_then(|a| self.objects[a].destination);
                if obj_dist.is_some() {
                    self.objects[LOC_PLAYER].location = obj_dist;
                    "OK.\n".to_string() + &self.do_look("")
                } else {
                    let obj_desc = obj_opt.map(|a| self.objects[a].description.clone());
                    obj_desc.unwrap_or("Invalid command!!\n".to_string())
                }
            }
        }
    }

    /// Player drops the specified object
    pub fn do_drop(&mut self, noun: &String) -> String {
        let (output, object_index) =
            self.get_possession(Some(LOC_PLAYER), Command::Drop("drop".to_string()), noun);

        let player_loc = self.objects[LOC_PLAYER].location;
        output + self.move_object(object_index, player_loc).as_str()
    }

    /// Player consumes the specified object
    pub fn do_consume(&mut self, object: Option<usize>) -> String {
        let heal = self.objects[object.unwrap()].health.unwrap_or(0);
        let mut player_health = self.objects[LOC_PLAYER].health.unwrap_or(0);
        if player_health == 100 {
            "You are already at full health".to_string()
        } else {
            self.objects[LOC_PLAYER].health = Some(
                self.objects[LOC_PLAYER]
                    .health
                    .map(|h| h + heal)
                    .unwrap_or(0),
            );
            player_health = self.objects[LOC_PLAYER].health.unwrap_or(0);
            if player_health > 100 {
                self.objects[LOC_PLAYER].health = Some(100);
            }
            self.objects[object.unwrap()].location = None;
            "You have consumed the item. Your health has increased to ".to_string()
                + &self.objects[LOC_PLAYER].health.unwrap_or(0).to_string()
                + "\n"
        }
    }

    /// Player gets the specified object
    pub fn do_get(&mut self, noun: &String) -> String {
        let (output, obj_opt) = self.object_visible(noun);
        let obj_item = obj_opt.map(|a| self.objects[a].item).unwrap_or(false);
        let player_to_obj = self.get_distance(Some(LOC_PLAYER), obj_opt);
        let obj_consumable = obj_opt.map(|a| self.objects[a].consumable).unwrap_or(false);

        match (player_to_obj, obj_opt, obj_item, obj_consumable) {
            (Distance::Player, _, _, _) => output + "Invalid!! You cannot get that!!",
            (Distance::Held, Some(obj_index), true, _) => {
                output
                    + &format!(
                        "You already have: {}.\n",
                        self.objects[obj_index].description
                    )
            }
            (Distance::OverThere, _, true, _) => output + "The item is not here. Try elsewhere!!\n",
            (Distance::OverThere, _, false, false) => output + "You cannot get that!!\n",
            (Distance::Here, _, false, false) => output + "You cannot get that!!\n",
            (Distance::Unknown, _, false, false) => output,
            (Distance::Here, _, true, true) => self.do_consume(obj_opt),
            _ => self.move_object(obj_opt, Some(LOC_PLAYER)),
        }
    }

    /// Player checks the inventory
    pub fn do_inventory(&self) -> String {
        let (list_string, count) = self.list_objects(LOC_PLAYER);
        if count == 0 {
            "You currently do not have anything in your inventory.\n".to_string()
        } else {
            list_string
        }
    }

    /// Returns true or false depending on if the object is contained by another object
    pub fn is_containing(&self, container: Option<usize>, object: Option<usize>) -> bool {
        object.is_some() && (object.and_then(|a| self.objects[a].location) == container)
    }

    /// Returns the distance of one object in relation to another object
    pub fn get_distance(&self, from: Option<usize>, to: Option<usize>) -> Distance {
        let from_loc = from.and_then(|a| self.objects[a].location);

        if to.is_none() {
            Distance::Unknown
        } else if to == from {
            Distance::Player
        } else if self.is_containing(from, to) {
            Distance::Held
        } else if self.is_containing(to, from) {
            Distance::Location
        } else if from_loc.is_some() && self.is_containing(from_loc, to) {
            Distance::Here
        } else if self.passage_index(from_loc, to).is_some() {
            Distance::OverThere
        } else {
            Distance::NotHere
        }
    }

    /// Returns the index of the object if it is visible
    pub fn describe_move(&self, obj_opt: Option<usize>, to: Option<usize>) -> String {
        let obj_loc = obj_opt.and_then(|a| self.objects[a].location);
        let player_loc = self.objects[LOC_PLAYER].location;

        match (obj_opt, obj_loc, to, player_loc) {
            (Some(obj_opt_idx), _, Some(to_idx), Some(player_loc_idx))
                if to_idx == player_loc_idx =>
            {
                format!("You have dropped {}.\n", self.objects[obj_opt_idx].label[0])
            }
            (Some(obj_opt_idx), _, Some(to_idx), _) if to_idx != LOC_PLAYER => {
                format!(
                    "You put {} in {}.\n",
                    self.objects[obj_opt_idx].label[0], self.objects[to_idx].label[0]
                )
            }
            (Some(obj_opt_idx), Some(obj_loc_idx), _, Some(player_loc_idx))
                if obj_loc_idx == player_loc_idx =>
            {
                format!("You pick up the {}.\n", self.objects[obj_opt_idx].label[0])
            }
            (Some(obj_opt_idx), Some(obj_loc_idx), _, _) => format!(
                "You got {} from {}.\n",
                self.objects[obj_opt_idx].label[0], self.objects[obj_loc_idx].label[0]
            ),
            // This arm should never get hit.
            (None, _, _, _) | (_, None, _, _) => "Please you have to drop something.\n".to_string(),
        }
    }

    /// Moves the object to the specified location
    pub fn move_object(&mut self, obj_opt: Option<usize>, to: Option<usize>) -> String {
        let obj_loc = obj_opt.and_then(|a| self.objects[a].location);

        match (obj_opt, obj_loc, to) {
            (None, _, _) => "".to_string(),
            (Some(_), _, None) => "No one is present here to give.\n".to_string(),
            (Some(_), None, Some(_)) => "You cannot get that!!\n".to_string(),
            (Some(obj_idx), Some(_), Some(to_idx)) => {
                let output = self.describe_move(obj_opt, to);
                self.objects[obj_idx].location = Some(to_idx);
                output
            }
        }
    }

    /// Gets the index of the passage if visible
    fn passage_index(&self, from: Option<usize>, to: Option<usize>) -> Option<usize> {
        let mut result: Option<usize> = None;

        match (from, to) {
            (Some(from), Some(to)) => {
                for (pos, object) in self.objects.iter().enumerate() {
                    let obj_loc = object.location;
                    let obj_dest = object.destination;
                    match (obj_loc, obj_dest) {
                        (Some(location), Some(destination))
                            if location == from && destination == to =>
                        {
                            result = Some(pos);
                            break;
                        }
                        _ => continue,
                    }
                }
                result
            }
            _ => result,
        }
    }

    /// Returns the index of the object if it is visible
    pub fn get_possession(
        &mut self,
        from: Option<usize>,
        command: Command,
        noun: &String,
    ) -> (String, Option<usize>) {
        let object_held = self.object_index(noun, from, Distance::Held);
        let object_not_here = self.object_index(noun, from, Distance::NotHere);

        match (from, object_held, object_not_here) {
            (None, _, _) => (
                format!("I don't understand what is needed {command}.\n"),
                None,
            ),
            (Some(_), AmbiguousOption::None, AmbiguousOption::None) => (
                format!("Please use correct command for: {}.\n", command),
                None,
            ),
            (Some(from), AmbiguousOption::None, _) if from == LOC_PLAYER => {
                (format!("You are not holding any {}.\n", noun), None)
            }
            (Some(from), AmbiguousOption::Some(object), _) if object == from => (
                format!(
                    "It is illegal to do this: {}.\n",
                    self.objects[object].label[0]
                ),
                None,
            ),
            (Some(_), AmbiguousOption::Ambiguous, _) => (
                format!(
                    "Please be more specific about which {} you want to {}.\n",
                    noun, command
                ),
                None,
            ),
            (Some(_), AmbiguousOption::Some(object_held), _) => ("".to_string(), Some(object_held)),
            (Some(_), AmbiguousOption::None, AmbiguousOption::Some(_))
            | (Some(_), AmbiguousOption::None, AmbiguousOption::Ambiguous) => {
                (format!("You don't see any {} here.\n", noun), None)
            }
        }
    }

    /// Returns player's location
    pub fn player_here(&self) -> Option<usize> {
        let mut player_loc: Option<usize> = None;

        for (pos, object) in self.objects.iter().enumerate() {
            match (pos, object.location) {
                (_, obj_loc) if (obj_loc == self.objects[LOC_PLAYER].location) => {
                    player_loc = Some(pos);
                    break;
                }
                _ => continue,
            }
        }

        player_loc
    }

    pub fn display_help(&self) -> String {
        "Available commands are\n
        look\n
        attack <enemy name>\n
        go <location>\n
        get <item name>\n
        drop <item name>\n
        inventory \n
        map \n
        quit\n
        help\n"
            .to_string()
    }

    pub fn display_locations(&self) -> String {
        let mut result = String::new();
        result += "Available locations:\n";
        let mut destinations = std::collections::HashSet::new();

        for object in &self.objects {
            if let Some(destination) = object.destination {
                destinations.insert(destination);
            }
        }

        for (index, object) in self.objects.iter().enumerate() {
            //let location = &self.objects[locations];
            if destinations.contains(&index) {
                //println!("{}: {}", index, object.label[0]);
                result += &format!("{}: {}\n", index, object.label[0]);
            }
        }
        result
    }
}

/// Default implementation for World
impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// Function that parses user's commands into a verb and a noun
pub fn parse(input: String) -> Command {
    let input = input.to_lowercase();
    let mut split_input = input.split_whitespace();

    let verb = split_input.next().unwrap_or_default().to_string();
    let noun = split_input.fold("".to_string(), |accum, item| {
        if accum.is_empty() {
            accum + item
        } else {
            accum + " " + item
        }
    });

    match verb.as_str() {
        "look" => Command::Look(noun),
        "go" => Command::Go(noun),
        "quit" => Command::Quit,
        "attack" => Command::Attack(noun),
        "drop" => Command::Drop(noun),
        "get" => Command::Get(noun),
        "help" => Command::Help,
        "inventory" => Command::Inventory,
        "map" => Command::Map,
        _ => Command::Unknown(input.trim().to_string()),
    }
}

/// Function that takes user's input
pub fn get_input() -> Command {
    print!("\n> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    parse(input)
}

/// Function to update the screen
pub fn update_screen(output: String) {
    for c in output.chars() {
        print!("{}", c);
        stdout().flush().unwrap(); // Flush the output to make it appear immediately
        thread::sleep(Duration::from_millis(25)); // Delay between characters
    }
}
