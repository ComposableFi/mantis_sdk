import subprocess
import schedule
import time

# List of commands to execute
commands = [
    "cargo run -- ethereum 0x4d224452801aced8b2f0aebe155379bb5d594381 6000000000000000000 0x6b175474e89094c44da98b954eedeac495271d0f 1 3600",
    "cargo run -- ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 3000000000000000 0x853d955acef822db058eb8505911ed77f175b99e 1 3600",
    "cargo run -- ethereum 0x6985884c4392d348587b19cb9eaaf157f13271cd 2000000000000000000 0xcf0c122c6b73ff809c693db761e7baebe62b6a2e 1 3600",
    "cargo run -- ethereum 0x2260fac5e5542a773aa44fbcfedf7c193bc2c599 11000 0x9f8f72aa9304c8b593d555f12ef6589cc3a579a2 1 3600",
    "cargo run -- ethereum 0x6b175474e89094c44da98b954eedeac495271d0f 8000000000000000000 0x5a98fcbea516cf06857215779fd812ca3bef1b32 1 3600",
    "cargo run -- ethereum 0x467719aD09025FcC6cF6F8311755809d45a5E5f3 9000000 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 1 3600",
]

# Function to run the next command in the list
def run_commands_in_sequence():
    for command in commands:
        try:
            print(f"Running command: {command}")
            result = subprocess.run(command, shell=True, check=True, capture_output=True, text=True)
            print(f"Command Output: {result.stdout}")
        except subprocess.CalledProcessError as e:
            print(f"Error executing command: {e.stderr}")
        # Wait 1 hour (3600 seconds) before running the next command
        time.sleep(7200)

# Schedule the task to run in sequence every hour
schedule.every().hour.do(run_commands_in_sequence)

if __name__ == "__main__":
    print("Scheduler started. Running commands in sequence every hour.")
    run_commands_in_sequence()  # Start the sequence immediately
    while True:
        schedule.run_pending()
        time.sleep(1)

