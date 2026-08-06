use std::fs;

use serenity::model::prelude::{GuildId, UserId};

pub fn get_secret(key: &str) -> Secret {
    let file = fs::read_to_string("./secrets.csv").expect("file unable to read");

    for s in file.lines() {
        let split: Vec<&str> = s.split(",").collect();

        if split[1].to_uppercase().eq(&key.to_uppercase()) {
            //this is the correct secret
            let sec_struct: Secret = Secret {
                name: split[0].to_owned(),
                key: split[1].to_owned(),
                value: split[2..].join(",")
            };
            return sec_struct;
        }
    }

    //nothing matched return blank
    let blank: Secret = Secret {
        name: String::from("none"),
        key: String::from("none"),
        value: String::from("none"),
    };
    blank
}

pub fn admin_server() -> Option<GuildId> {
    get_secret("ADMIN_SERVER")
        .value
        .parse::<u64>()
        .ok()
        .map(GuildId::from)
}

pub fn admin_list() -> Vec<UserId> {
    get_secret("ADMIN_LIST")
        .value
        .split(',')
        .filter_map(|s| s.trim().parse::<u64>().ok().map(UserId::from))
        .collect()
}

pub fn is_admin_user(user_id: UserId) -> bool {
    admin_list().contains(&user_id)
}

pub struct Secret {
    pub name: String,
    pub key: String,
    pub value: String,
}
