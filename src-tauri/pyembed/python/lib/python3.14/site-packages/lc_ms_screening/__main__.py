from anyio.from_thread import start_blocking_portal

from lc_ms_screening.ext_mod import pytauri as _pytauri
from .bridge import commands

context_factory = _pytauri.context_factory
builder_factory = _pytauri.builder_factory


def main() -> int:
    with start_blocking_portal("asyncio") as portal:
        app = builder_factory().build(
            context=context_factory(),
            invoke_handler=commands.generate_handler(portal),
        )
        return app.run_return()


if __name__ == "__main__":
    raise SystemExit(main())
