import tomllib


class Plugin:
    config = None

    def __init__(self, config) -> None:
        self.config = tomllib.loads(config)

    def exec(self, data):
        with open(self.config["path"], "ba") as f:
            f.write(data)
