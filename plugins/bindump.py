import tomllib
import logging


class Plugin:
    config: dict[str, str] = {}

    def __init__(self, config: str) -> None:
        self.config = tomllib.loads(config)

    def exec(self, data: bytes) -> None:
        logging.info(f"Dumping {len(data)} bytes to {self.config['path']}")
        with open(self.config["path"], "ba") as f:
            f.write(data)
