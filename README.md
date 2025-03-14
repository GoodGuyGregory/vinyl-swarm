# Vinyl Swarm

Collecting vinyl is an adventure, but with so much out there, finding everything you want can be overwhelming. Another challenge? As your collection grows, so does the difficulty of keeping track—when you hit 100+ records, knowing what you own becomes a task in itself. I haven’t quite reached that number yet, but I know I will. That’s why I built this project. 

On top of that, hunting for records isn't just about what you already own—it’s about discovering new places, supporting local merchants, and building a community of fellow collectors. With Vinyl Swarm, you can search for friends, share collections, and track down hidden gems together. Whether you’re digging through crates at your favorite shop or mapping out new stores to explore, this app makes it easier to connect, collect, and celebrate the vinyl experience.

This backend Axum application, written in Rust, helps categorize and search your collection, so instead of racking your brain for what’s missing, you can let the system do the work.

My hope is that this API makes finding vinyl-collecting friends easier, helps you keep track of your collection, and, most importantly, helps you finally snag those rarities you’ve been after. Now get out there and form your own Vinyl Swarm. 🎶

### Features

* **User Collections:** Store your personal record collection details with the world and show off your rare finds. This also helps friends determine what records they might want to take off your hands. Collections can be viewed by others and they make great conversation starters.

* **Records:** Records are representations of actual vinyl releases. The Records collection will be stored in the database for anyone to look at and add to their collection. This is a running list of all records for every user. It's community driven.

* **Record Stores:** What app wouldn't be complete without the best havens for searching for the your next favorite artist's LP. The API allows for accessing Record Stores throughout the US, offering a description of the store and it's city state, as well as the website to browse stock.

## API Documentation

[Vinyl Swarm API](https://documenter.getpostman.com/view/5839344/2sAYk7SizX) Keep in mind the documentation won't operate correctly unless the project is build and running. 

------  

# Getting Started


## Ensure PostgreSQL is Installed

if you have not done so [install postgresql](https://www.postgresql.org/download/) the database backend is configured to work with postgres through `SQLX`. This is leveraged to interface with postgres through the Rust API backend.

**What is SQLX?**

>   SQLX is a Rust library that provides a type-safe SQL library for Rust applications. It takes the static query strings and returns the entire SQL result set as a Rust type. It's a pure Rust library that is used for asynchronous database   handling in Rust.  

[SQLX Project](https://sqlx.dev/article/A_Beginners_Guide_to_SQLX_Getting_Started.html)

## Clone the Repo

clone this repo somewhere safe and access the code locally.

## Cargo Build

after cloning `change directory` into the project then build the project using the following dependencies with these features 

```bash
# change directory to the project
cd vinyl-swarm/vinyl-swarm

# build the project 
cargo build
```

**Dependencies Required**

```toml
[dependencies]
axum = "0.8.1"
bcrypt = "0.17.0"
bigdecimal = { version = "0.4", features = ["serde"] }
chrono = { version = "0.4.40", features = ["serde"] }
dotenv = "0.15.0"
serde = { version = "1.0.218", features = ["derive"] }
serde_json = "1.0.140"
sqlx = { version = "0.8.3", features = ["runtime-async-std-native-tls", "postgres", "chrono", "uuid", "bigdecimal"] }
tokio = { version = "1.43.0", features = ["full"] }
tower-http = { version = "0.6.2", features = ["cors"] }
uuid = { version = "1.15.1", features = ["serde", "v4"] }
```

## Initialize Database

After building the project you're ready to add the initial database files to start adding your personal collection. 

**Create Database Configuration**

point the project to your newly created database and supply the connection string inside of a `.env` file at the `root` of the project.

**Project Structure Below**

place the  `.env` in the same location.

```tree
├── Cargo.lock
├── Cargo.toml
├── migrations
│   ├── 20250308183724_init.down.sql
│   └── 20250308183724_init.up.sql
├── src
│   ├── handlers
│   │   ├── mod.rs
│   │   ├── record_stores.rs
│   │   ├── records.rs
│   │   └── users.rs
│   ├── main.rs
│   ├── models
│   │   ├── mod.rs
│   │   ├── record.rs
│   │   ├── store.rs
│   │   └── user.rs
│   └── routes
│       ├── mod.rs
│       └── router.rs
├── .env
```

**Contents of ENV**

if you haven't already done so create a `vinyl_swarm` database with a `user` and `password` for this database then add the configuration details to your `.env` exposure of your database in postgres might be on a different port other than `localhost:5432` check your configurations within your database for trouble shooting.

```
DATABASE_URL=postgresql://<database_username>:<database_user_password>@localhost:5432/vinyl_swarm
```

**Install the SQLX CLI**

this will be used to begin using the API

```bash
# used to run the initial migration
cargo install sqlx-cli
```

**Run Initial Migration**

once the `sqlx-cli` process is installed. ensure you're inside of the project code. 


```bash
# check directory ensure `<cwd>/vinyl-swarm/vinyl-swarm
pwd

# once you've confirmed your inside the project
# run the initial migration with sqlx cli
sqlx migrate run
```

the `sqlx migrate run` will install all of your initial database tables, and collections used for the project. if you want to check out the relational database configuration checkout the [initial migration script](./vinyl-swarm/migrations/20250308183724_init.up.sql)

## Run the Project

from the terminal start up the application, but ensure the current working directory still points to `vinyl-swarm/vinyl-swarm` 

```bash
cargo run   
```

**Open API Documentation**

[Vinyl Swarm API](https://documenter.getpostman.com/view/5839344/2sAYk7SizX) click the `Run in Postman` button if you're not familiar with `Curl Commands`



## Contact

GoodGuyGregory 🚲  
[greg@goodguygregory.com](mailto:greg@goodguygregory.com)


### License 

Copyright (c) 2011-2026 GitHub Inc.

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
