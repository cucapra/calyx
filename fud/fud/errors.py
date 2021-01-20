from pathlib import Path


class FudError(Exception):
    """
    An error caught by the FuTIL Driver.
    """

    def __init__(self, msg):
        super().__init__(msg)


class NoFile(FudError):
    def __init__(self, possible_dests=None):
        msg = "No filename or type provided for exec."
        if possible_dests is not None:
            dests = ",".join(map(lambda e: e.dest, possible_dests))
            msg += f"\nPossible destination stages: [{dests}]"
        super().__init__(msg)


class UnknownExtension(FudError):
    """
    The provided extension does not correspond to any known stage.
    Thrown when the implicit stage discovery mechanism fails.
    """

    def __init__(self, filename):
        path = Path(filename)
        ext = path.suffix
        super().__init__(
            f"`{ext}' does not correspond to any known stage. Please provide an explicit stage using --to or --from."
        )


class UnsetConfiguration(FudError):
    """
    Execution of a stage requires some configuration. Thrown when missing configuration.
    """

    def __init__(self, path):
        path_str = ".".join(path)
        msg = f"'{path_str}' is not set. Use `fud config {path_str} <val>` to set it."
        super().__init__(msg)


class MissingDynamicConfiguration(FudError):
    """
    Execution of a stage requires dynamic configuration. Thrown when such a
    configuration is missing.
    """

    def __init__(self, variable):
        msg = (
            "Provide an input file or "
            + f"`{variable}' needs to be set. "
            + "Use the runtime configuration flag to provide a value: '-s {variable} <value>'."
        )
        super().__init__(msg)


class NoPathFound(FudError):
    """
    There is no way to convert file in input stage to the given output stage.
    """

    def __init__(self, source, destination):
        msg = f"No way to convert input in stage `{source}' to stage `{destination}'."
        super().__init__(msg)


class TrivialPath(FudError):
    """
    The execution doesn't run any stages. Likely a user mistake.
    """

    def __init__(self, stage):
        msg = f"The exection starts and ends at the same stage `{stage}'. This is likely an error."
        super().__init__(msg)


class SourceConversion(FudError):
    """
    Can't convert to a particular source type.
    """

    def __init__(self, source_t, dst_t):
        msg = f"Can't convert from {source_t} to {dst_t}"
        super().__init__(msg)


class RemoteLibsNotInstalled(FudError):
    """
    Libraries needed for remote use of tools are not installed.
    """

    def __init__(self):
        msg = f"Attempted to use remote features without both [paramiko, scp] installed. Install them and try again."
        super().__init__(msg)
