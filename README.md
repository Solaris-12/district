# DISTRICT Server

> [!NOTE]  
> This project is no longer actively developed by the devs. They can provide security updates (when asked to) and your contributions are welcome.

DISTRICT server is a powerful modular backend server designed for connecting (multiple) game servers with Discord and a database.

Coded in Rust, it offers speed through Rocket HTTP and WebSocket servers, along with reliable databases provided by SqLite3. 

This project was originally developed for SCP: Secret Laboratory server 'Solaris-12', but it now offers features that can be used elsewhere.

## Key Features

- ⚡**Fast**: Utilizes Rust's performance capabilities and Rocket HTTP for fast response times.
- 🦾**Reliable**: SqLite3 provides a reliable database backend.
- 🔌**Connects with Discord**: Seamlessly integrates with Discord for communication and server management.
- 📊**Leaderboards**: Tracking player progress and statistics.
- ⚙️**Minimal maintenance**: Designed for stability with minimal maintenance.

## Roadmap

- 💻**Commands**: (Half done), Create discord commands for easier managing of servers.
- 👔**More Customizability**: Offering greater flexibility for customization within config.
- 🤖**Auto Commands**: Implementing automated commands for routine tasks or on-event.
- 📦**Lua Plugin System**: Introducing a Lua plugin system for extending functionality.

## Deployment

0. Requirements:
  - Rust
  - Cargo
  - Copy everything from `./resources/` to `./`
  - At least 1 Discord bot token

1. Build the project:
```sh
cargo build --release
```

_(If successfully built, files will be located at `./target/release`)_

2. Run the server:
```sh
./target/release/district_server
```

_(This should crash on startup, you have to setup the bot 'token' in generated `./config.json` file (under the main_bot>token) and the 'active_guild_id')_

3. Change configs

   - `./config.json` - For server configs
   - `./lang.json` - For translations from your game server to Discord
   - If you are looking for list of all options, look at [src/application/config](./src/application/config/config.rs)
   - Example server:

```json
{
  "id": 6452321,
  "name": "My Game Server",
  "channel_id": "768464464643413468",
  "bot": {
    "token": "SoM3Ver7S3cRetT0.k3nth8Th85toB3Sup13d",
    "active_guild_id": "5646464676465467",
    "commands": {
      "info_command": 64613468765144645,
      "send_command": null,
      "db_search": 65431354625965545,
    },
  },
}
```

4. Connection to the server
   - When the server is online, you can access it on the link [0.0.0.0:9005](http://0.0.0.0:9005/).
   - All the routes are listed at [src/application/routes/mod.rs](./src/application/routes/mod.rs#40).
   - If you wish to use the WebSocket, there are two routes (logs, stats); you can find them [src/application/routes/websocket/mod.rs](./src/application/routes/websocket/mod.rs#222).

5. Start the server

```sh
./target/release/district_server
```

## License

This project is open to everyone under the MIT License. For more information, see the [LICENSE](./LICENSE) file.
