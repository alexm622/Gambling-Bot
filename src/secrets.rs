use std::fs;

pub async fn get_secret(key: &str) -> Secret {
    let file = fs::read_to_string("./secrets.csv").expect("file unable to read");

    for s in file.lines() {
        let split: Vec<&str> = s.split(",").collect();

        if split[1].to_uppercase().eq(&key.to_uppercase()) {
            //this is the correct secret
            let sec_struct: Secret = Secret {
                name: split[0].to_owned(),
                key: split[1].to_owned(),
                value: split[2].to_owned(),
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
    return blank;
}

pub struct Secret {
    pub name: String,
    pub key: String,
    pub value: String,
}
