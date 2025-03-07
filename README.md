# Mystiko.Network.Roller - The Rollup Layer for Mystiko.Network

[![build status](https://github.com/mystikonetwork/mystiko-roller/actions/workflows/build.yml/badge.svg)](https://github.com/mystikonetwork/mystiko-roller/actions/workflows/build.yml)

Mystiko.Network.Roller is a rollup layer for Mystiko.Network. It provides the following features:

- Monitor rollup status of Mystiko.Network.
- Rollup transactions of Mystiko.Network.

## Roller Environment

Following the list of environment variables that must set when running Roller:

```Shell
MYSTIKO_ROLLER.HOME_PATH: roller worker path, default is `/home/mystiko-miner/roller`.
MYSTIKO_ROLLER.PRIVATE_KEY: roller rollup account private key.
MYSTIKO_ROLLER.TOKEN_PRICE_API_KEY: coinmarketcap API key to get token price.
```

Following the list of path environment variables that can be set when running Roller, if not set Roller will use the
`MYSTIKO_ROLLER.HOME_PATH`:

```Shell
MYSTIKO_ROLLER.CONFIG_PATH: roller config path.
MYSTIKO_ROLLER.DATA_PATH: roller data path.
MYSTIKO_ROLLER.CIRCUITS_PATH: roller circuits files path.
```

Roller support environment variables to set the configuration. the prefix of the environment variables is
`MYSTIKO_ROLLER`.

## Running Roller via Shell

To run Roller directly from your terminal, follow these steps:

1. Open your terminal.
2. Navigate to the project's root directory.
3. Build the project using the following command:

```Shell
cargo build --release
```

4. Prepare the configuration file `roller.json`. (Refer to the [Configuration](./config) section for more information.)
   put `roller.json` in the `MYSTIKO_ROLLER.CONFIG_PATH` or `MYSTIKO_ROLLER.HOME_PATH/config` directory.
5. Execute the commands to start Roller.

```Shell
MYSTIKO_ROLLER.HOME_PATH=$(pwd) \
MYSTIKO_ROLLER.PRIVATE_KEY=0x... \
MYSTIKO_ROLLER.TOKEN_PRICE_API_KEY=... \
./target/release/roller
```

6. Can execute the commands to start Roller with environment variables to override roller.json config.

```Shell
MYSTIKO_ROLLER.HOME_PATH=$(pwd) \
MYSTIKO_ROLLER.PRIVATE_KEY=0x... \
MYSTIKO_ROLLER.TOKEN_PRICE_API_KEY=... \
MYSTIKO_ROLLER.MEMORY_DB=false \
MYSTIKO_ROLLER.SCHEDULER.SCHEDULE_INTERVAL_MS=10 \
MYSTIKO_ROLLER.ROLLUP.MAX_ROLLUP_SIZE=64 \
./target/release/roller
```

## Running Roller with Docker

Ensure Docker is installed and then run Roller inside a container. Use the following steps:

1. Build the Docker image (if needed).

```Shell
cargo build --release
docker build -t mystikonetwork/rollup-miner -f docker/Dockerfile .
```

3. Prepare the configuration file `roller.json`, put `roller.json` in the `$(pwd)/config` directory.
2. Start the Docker container with the appropriate configurations.

```Shell
docker run -d --name roller \
  -v "$(pwd)/config:/home/mystiko-miner/roller/config" \
  -v "$(pwd)/data:/home/mystiko-miner/roller/data" \
  -v "$(pwd)/circuits:/home/mystiko-miner/roller/circuits" \
  --env MYSTIKO_ROLLER.PRIVATE_KEY=0x... \
  --env MYSTIKO_ROLLER.TOKEN_PRICE_API_KEY=.... \
  mystikonetwork/rollup-miner
```