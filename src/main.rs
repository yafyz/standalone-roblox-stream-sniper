use std::path;
use std::process;
use std::io;
use std::io::Write;
use std::fs;
use std::io::prelude::Read;
extern crate ureq;
extern crate dirs;
extern crate serde_json;

fn get_xcsrf_token(cookie: &str) -> String {
    let resp: ureq::Response = ureq::post("https://auth.roblox.com/v1/authentication-ticket/") // send request
        .set("Cookie", &format!("{}{}", ".ROBLOSECURITY=", &cookie))
        .set("Content-Length", "0")
        .call();
    resp.header("X-CSRF-TOKEN").unwrap().to_owned() // return the token, if its not there, we gon panic
}

fn get_auth_ticket(cookie: &str) -> String {
    let resp: ureq::Response = ureq::post("https://auth.roblox.com/v1/authentication-ticket/") // send request
        .set("Cookie", &format!("{}{}", ".ROBLOSECURITY=", &cookie))
        .set("X-CSRF-TOKEN", &get_xcsrf_token(&cookie))
        .set("Referer", "https://www.roblox.com/") // yeah roblox checks for referer when you request that
        .set("Content-Length", "0")
        .call();
    if resp.ok() {
        resp.header("RBX-Authentication-Ticket").unwrap().to_owned() // maybe should have put here a match case, this code is so inconsistent
    } else {
        println!("{}",&resp.into_string().unwrap()); // ey something fucked up if we are here
        unimplemented!()
    }
}

fn get_roblox_executable(name: &str) -> Option<path::PathBuf> { // oh no, lets just not even comment on this
    let path: path::PathBuf = dirs::cache_dir().unwrap().join("Roblox\\Versions");
    for entry in path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            let entry_path: path::PathBuf = entry.path();
            if entry_path.is_dir() {
                for item in entry_path.read_dir().expect("read_dir call failed") {
                    if let Ok(item) = item {
                        let item_path: path::PathBuf = item.path();
                        if item_path.file_name().unwrap() == name {
                            return Some(item_path) // return the executable path
                        }
                    }
                }
            }
        }
    }
    None // oh fuck we got nothing
}

fn launch_game_instance(cookie: &str, placeid: &str, jobid: Option<&str>) {
    let auth: String = get_auth_ticket(&cookie); // get the auth
    let place_launcher: String; // prepare the variable
    match jobid {
        None => place_launcher = {
            println!("Join game attempt on {}", &placeid);
            format!("{}{}{}","https://assetgame.roblox.com/game/PlaceLauncher.ashx?request=RequestGame&placeId=",&placeid,"&isPlayTogetherGame=false")
        },
        Some(x) => place_launcher = {
            println!("Join game instance attempt on {} ({})", &placeid, &x);
            format!("{}{}{}{}{}","https://assetgame.roblox.com/game/PlaceLauncher.ashx?request=RequestGameJob&placeId=",&placeid,"&gameId=", &x,"&isPlayTogetherGame=false")
        }
    }
    println!("Auth Token: {}...", &auth[..40]);
    print!("Finding Roblox executable... ");
    let path: path::PathBuf = get_roblox_executable("RobloxPlayerBeta.exe").unwrap(); // get the executable
    println!("{}", path.file_name().unwrap().to_str().unwrap());
    let _x = process::Command::new(&path) // launch roblex
        .arg("--play")
        .args(&["-a", "https://www.roblox.com/Login/Negotiate.ashx"])
        .args(&["-t", &auth])
        .args(&["-j", &place_launcher])
        .spawn();
    println!("Joining game..."); // profit
}

fn find_player(cookie: &str,placeid:  &str, username: &str) -> Option<String> {
    println!("Getting {}'s userid", username);
    let resp: ureq::Response = ureq::get("https://api.roblox.com/users/get-by-username") // get user id
        .query("username", &username)
        .call();
    if resp.ok() { // check if status is 200
        let resp_json: ureq::SerdeValue = resp.into_json().unwrap(); // make json from resp
        if resp_json["success"] == "false" { // check if the request suceeded
            unimplemented!()
        }
        let head_resp: ureq::Response = ureq::get("https://thumbnails.roblox.com/v1/users/avatar-headshot") // get user's thumbnail
            .query("size", "48x48")
            .query("format", "png")
            .query("userIds", &resp_json["Id"].to_string())
            .call();
        if head_resp.ok() { // check if status is 200
            let image_json: ureq::SerdeValue = head_resp.into_json().unwrap(); //make json from head_resp
            if image_json["data"][0]["state"] == "Completed" { // check if the request suceeded
                let image_url: &str = &image_json["data"][0]["imageUrl"].to_string().to_owned(); // extract the image url
                let mut start_index: u32 = 0; // page index
                let mut total_size: u32; // total servers
                loop {
                    let game_instances_resp: ureq::SerdeValue = ureq::get(&format!("{}{}{}{}","https://www.roblox.com/games/getgameinstancesjson?placeId=",&placeid,"&startIndex=",start_index))
                        .set("Cookie", &format!("{}{}", ".ROBLOSECURITY=", cookie)) // get game instances
                        .call()
                        .into_json() // response into json
                        .unwrap();
                    total_size = game_instances_resp["TotalCollectionSize"].to_string().parse::<u32>().unwrap(); // get total ammount of servers
                    println!("{}/{}", start_index, total_size);
                    let servers: String = game_instances_resp["Collection"].to_string(); // make a string out of that
                    let serde_server_value: serde_json::Deserializer<serde_json::de::StrRead<'_>> = serde_json::Deserializer::from_str(&servers); // create a Deserializer
                    for value in serde_server_value.into_iter::<Vec<serde_json::Value>>().next().unwrap().unwrap() { // some jank iterator i made which i dont even know how works
                        let stringed: String = value.to_string(); // to string dat
                        println!("{}", value["Guid"]);
                        if stringed.find(&image_url) != None { // check if the url is in the string
                            let temp: String = value["Guid"].to_string();
                            return Some(temp[1..temp.len() - 1].to_owned()) // return the server guid
                        }
                    }
                    start_index += 9; // advance to next page
                    if start_index >= total_size { //if next page is out of range, exit, player not found
                        break
                    } 
                }
                None // here is just lots of returning, nothing interesting
            } else {
                println!("{}", image_json.to_string());
                None
            }
        } else {
            println!("{}", head_resp.into_string().unwrap());
            None
        }
    } else {
        println!("{}", resp.into_string().unwrap());
        None
    }
}

fn main() -> io::Result<()> {
    let mut cookie: String = String::new();
    let mut cookie_file: fs::File = fs::File::open("cookie")?;
    cookie_file.read_to_string(&mut cookie)?;
    let mut game_id: String = String::new();// = "606849621";
    let mut person: String = String::new();
    print!("Place id: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut game_id)?;
    game_id = game_id.split_whitespace().next().unwrap().to_owned();
    print!("Username: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut person)?;
    person = person.split_whitespace().next().unwrap().to_owned();
    let server_job_id: Option<String> = find_player(&cookie, &game_id, &person);
    match server_job_id {
        None => println!("{}", "User not found"),
        Some(x) => launch_game_instance(&cookie, &game_id, Some(&x))
    }
    io::stdin().read_line(&mut "".to_owned())?;
    Ok(())
}
