from fud.stages import Stage, Step, SourceType
from ..utils import unwrap_or


class FutilStage(Stage):
    def __init__(self, config, destination, flags, desc):
        self.flags = flags
        super().__init__('futil', destination, config, desc)

    def _define(self):
        main = Step(SourceType.File)
        main.set_cmd(' '.join([
            self.cmd,
            '-l', self.global_config["futil_directory"],
            self.flags,
            unwrap_or(self.stage_config['flags'], ''),
            '{ctx[last]}'

        ]))
        main.last_context = {
            'last': '--force-color'
        }
        return [main]
