import argparse
import subprocess
import time

import requests

BASE_URL = "http://127.0.0.1:8000"
BIN_PATH = "./bimc"


def get_machine_id(token: str, name: str) -> int:
    data = {
        "token": token,
        "name": name,
    }
    r = requests.post(f"{BASE_URL}/api/machines/", json=data)
    if not r.ok:
        return 0
    return int(r.json()["id"])


class Runner:
    token = None
    machine_id = None

    def __init__(self, token: str, machine_id: int) -> None:
        self.token = token
        self.machine_id = machine_id

    def get_tasks(self):
        r = requests.get(
            f"{BASE_URL}/api/tasks/?token={self.token}&machine_id={self.machine_id}&status=Ready")
        if not r.ok:
            return []
        res = r.json()
        return res

    def finish_task(self, task, output: str):
        upload, upload_status, download, download_status, latency, jitter = output.split(
            ",")
        task_id = task["id"]

        data = {
            "download": 0,
            "download_status": download_status.strip(),
            "upload": 0,
            "upload_status": upload_status.strip(),
            "latency": 0,
            "jitter": 0
        }

        try:
            data = {
                "download": float(download.strip()),
                "upload": float(upload.strip()),
                "latency": float(latency.strip()),
                "jitter": float(jitter.strip())
            }
        except:
            pass

        requests.post(
            f"{BASE_URL}/api/tasks/{task_id}?token={self.token}", json=data)

    def run(self):
        while True:
            tasks = self.get_tasks()
            print(f"{len(tasks)} tasks")

            for task in tasks:
                server = task['server']
                args = [BIN_PATH, server["download_url"], server["upload_url"]]
                if server["ipv6"]:
                    args.append("-6")
                if server["multi"]:
                    args.append("-m")
                t = subprocess.run(args, capture_output=True)

                output = t.stdout.decode("utf-8")
                self.finish_task(task, output)

            if len(tasks) == 0:
                time.sleep(30)
            else:
                time.sleep(5)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("name")
    parser.add_argument("token")
    args = parser.parse_args()

    machine_id = get_machine_id(args.token, args.name)
    if machine_id > 0:
        runner = Runner(args.token, machine_id)
        runner.run()
    else:
        print("Invalid token or name")
