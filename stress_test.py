import aiohttp
import asyncio
import ujson
import time


class TimerError(Exception):

    """A custom exception used to report errors in use of Timer class"""


class Timer:

    def __init__(self):

        self._start_time = None

    def start(self):
        """Start a new timer"""

        if self._start_time is not None:

            raise TimerError(f"Timer is running. Use .stop() to stop it")

        self._start_time = time.perf_counter_ns()

    def stop(self):
        """Stop the timer, and report the elapsed time"""

        if self._start_time is None:

            raise TimerError(f"Timer is not running. Use .start() to start it")

        elapsed_time = time.perf_counter_ns() - self._start_time

        self._start_time = None

        print(f"Elapsed time: {elapsed_time} seconds")


collections = []

for i in range(0, 5000):
    collections.append({"key": f"test-{i}"})

'''
serialized_collections = []
for collection in collections:
    serialized_collections.push(ujson.dumps(collection))
'''

t = Timer()


async def main():
    t.start()
    async with aiohttp.ClientSession(json_serialize=ujson.dumps) as client:
        for collection in collections:
            await client.post("http://127.0.0.1:8088/collections", json=collection)
        t.stop()

if __name__ == "__main__":
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main())
