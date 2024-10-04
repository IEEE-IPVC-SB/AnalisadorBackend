# Python script to simulate sensor telemetry. Sends packets to a running server.

import requests
import struct
import time
import random
from datetime import datetime

url = "http://0.0.0.0:3000/water"

while True:
    ph = 7.0 + random.uniform(-0.6, 0.6)
    ph_timestamp = int(datetime.now().timestamp())
    tds = 500.0 + random.uniform(-60, 60)
    tds_timestamp = int(datetime.now().timestamp())
    packet_timestamp = int(datetime.now().timestamp())

    # The rust struct is `#[repr(C, packed)]`, as the actual sensor packets will be, so we need
    # to replicate that here. 
    packet = struct.pack('dqdqq', ph, ph_timestamp, tds, tds_timestamp, packet_timestamp)
    # "dqdqq" - double,ulong,double,ulong,ulong

    response = requests.post(url, data=packet)

    if response.status_code == 200:
        print("Packet sent successfully!")
    else:
        print(f"Response: {response.text}")

    # Wait for 3 seconds before sending the next packet
    time.sleep(1.4)
