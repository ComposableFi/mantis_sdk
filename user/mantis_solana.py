import subprocess
import schedule
import time

# List of commands to execute
commands = [
    "cargo run -- solana 150000000 rndrizKT3MK1iimdxRdWabcF7Zg7AR5T4nud4EkHBof Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB 1 3600",
    "cargo run -- solana 3000000 EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v 1 3600",
    "cargo run -- solana 7000000 JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL 1 3600",
    "cargo run -- solana 25000000 HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3 Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB 1 3600",
    "cargo run -- solana 35000000000 DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263 5z3EqYQo9HiCEs3R84RCDMu2n7anpDMxRhdK8PSWmrRC 1 3600",
    "cargo run -- solana 3000000 4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R A8C3xuqscfmyLrte3VmTqrAq8kgMASius9AFNANwpump 1 3600"
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
        time.sleep(3600)

# Schedule the task to run in sequence every hour
schedule.every().hour.do(run_commands_in_sequence)

if __name__ == "__main__":
    print("Scheduler started. Running commands in sequence every hour.")
    run_commands_in_sequence()  # Start the sequence immediately
    while True:
        schedule.run_pending()
        time.sleep(1)

