"""Register an account on Vaultwarden the way the web vault does.

Usage: python3 -I vw-register.py BASE_URL EMAIL MASTER_PASSWORD [TOTP_SECRET]
       python3 -I vw-register.py --totp TOTP_SECRET   (print a code bw can use now)

With TOTP_SECRET (base32), the account also gets authenticator-app two-step
login, the way dot-bw-setup expects a real account to have it.

Derives the master key (PBKDF2-SHA256, 600000 iterations), creates the
encrypted user key and RSA key pair, and calls the two-step registration
endpoints. With mail off, Vaultwarden returns the verification token from the
first call. Prints "registered" or "exists"; exits non-zero on any other answer.
"""

import base64
import hashlib
import hmac
import os
import struct
import sys
import time
import uuid

import requests
from cryptography.hazmat.primitives import hashes, padding, serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.primitives.kdf.hkdf import HKDFExpand

ITERATIONS = 600000


def b64(data):
    return base64.b64encode(data).decode()


def enc_string(data, enc_key, mac_key):
    """Bitwarden EncString type 2: AES-256-CBC + HMAC-SHA256."""
    iv = os.urandom(16)
    padder = padding.PKCS7(128).padder()
    padded = padder.update(data) + padder.finalize()
    encryptor = Cipher(algorithms.AES(enc_key), modes.CBC(iv)).encryptor()
    ct = encryptor.update(padded) + encryptor.finalize()
    mac = hmac.new(mac_key, iv + ct, hashlib.sha256).digest()
    return f"2.{b64(iv)}|{b64(ct)}|{b64(mac)}"


# Vaultwarden refuses a code from a time step it has already accepted (replay
# protection) but takes one step of drift, so each use takes the next unused
# step, recorded here, waiting only when two steps ahead would be needed.
STEP_FILE = "/tmp/dotfiles-test/totp-step"


def next_step():
    try:
        with open(STEP_FILE) as f:
            last = int(f.read().strip() or 0)
    except (OSError, ValueError):
        last = 0
    step = max(int(time.time() // 30), last + 1)
    while step > int(time.time() // 30) + 1:
        time.sleep(1)
    with open(STEP_FILE, "w") as f:
        f.write(str(step))
    try:
        os.chmod(STEP_FILE, 0o666)  # root seeds, kuba logs in
    except PermissionError:
        pass
    return step


def totp(secret):
    key = base64.b32decode(secret.upper() + "=" * (-len(secret) % 8))
    counter = struct.pack(">Q", next_step())
    digest = hmac.new(key, counter, hashlib.sha1).digest()
    offset = digest[-1] & 0x0F
    return "%06d" % ((struct.unpack(">I", digest[offset : offset + 4])[0] & 0x7FFFFFFF) % 1000000)


def enable_totp(base, email, master_hash, secret):
    r = requests.post(
        f"{base}/identity/connect/token",
        data={
            "grant_type": "password",
            "username": email,
            "password": master_hash,
            "scope": "api offline_access",
            "client_id": "web",
            "deviceType": "10",
            "deviceIdentifier": str(uuid.uuid4()),
            "deviceName": "dotfiles-test",
        },
        timeout=30,
    )
    if r.status_code != 200:
        sys.exit(f"token: {r.status_code} {r.text[:300]}")
    auth = {"Authorization": "Bearer " + r.json()["access_token"]}
    r = requests.post(
        f"{base}/api/two-factor/authenticator",
        json={"key": secret, "token": totp(secret), "masterPasswordHash": master_hash},
        headers=auth,
        timeout=30,
    )
    if r.status_code != 200:
        sys.exit(f"two-factor/authenticator: {r.status_code} {r.text[:300]}")


def main():
    if sys.argv[1] == "--totp":
        print(totp(sys.argv[2]))
        return
    base, email, password = sys.argv[1].rstrip("/"), sys.argv[2].lower(), sys.argv[3]
    secret = sys.argv[4] if len(sys.argv) > 4 else None
    master_key = hashlib.pbkdf2_hmac("sha256", password.encode(), email.encode(), ITERATIONS, 32)
    master_hash = b64(hashlib.pbkdf2_hmac("sha256", master_key, password.encode(), 1, 32))

    def stretch(info):
        return HKDFExpand(hashes.SHA256(), 32, info).derive(master_key)

    user_key = os.urandom(64)
    protected_key = enc_string(user_key, stretch(b"enc"), stretch(b"mac"))
    private = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    public_der = private.public_key().public_bytes(
        serialization.Encoding.DER, serialization.PublicFormat.SubjectPublicKeyInfo
    )
    private_der = private.private_bytes(
        serialization.Encoding.DER, serialization.PrivateFormat.PKCS8, serialization.NoEncryption()
    )

    r = requests.post(
        f"{base}/identity/accounts/register/send-verification-email",
        json={"email": email, "name": "Test", "receiveMarketingEmails": False},
        timeout=30,
    )
    if r.status_code == 400 and "already" in r.text.lower():
        print("exists")
        return
    r.raise_for_status()
    token = r.text.strip().strip('"')
    body = {
        "email": email,
        "name": "Test",
        "masterPasswordHash": master_hash,
        "masterPasswordHint": None,
        "key": protected_key,
        "kdf": 0,
        "kdfIterations": ITERATIONS,
        "keys": {"publicKey": b64(public_der), "encryptedPrivateKey": enc_string(private_der, user_key[:32], user_key[32:])},
        "emailVerificationToken": token,
    }
    r = requests.post(f"{base}/identity/accounts/register/finish", json=body, timeout=30)
    if r.status_code == 400 and "already" in r.text.lower():
        print("exists")
        return
    if r.status_code != 200:
        sys.exit(f"register/finish: {r.status_code} {r.text[:300]}")
    if secret:
        enable_totp(base, email, master_hash, secret)
        print("registered, authenticator two-step login on")
    else:
        print("registered")


if __name__ == "__main__":
    main()
