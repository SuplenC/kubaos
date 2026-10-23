#!/usr/bin/env bash
# vw-certs.sh DIR: a throwaway CA and a server certificate for "vw" (the
# Vaultwarden container), because bw and kubaos bitwarden only talk https.
# Runs once per volume (compose service vw-certs).
set -euo pipefail
dir="${1:?usage: vw-certs.sh DIR}"
if [[ -s $dir/cert.pem && -s $dir/key.pem && -s $dir/ca.pem ]]; then
  echo "certificates present in $dir"
  exit 0
fi
cd "$dir"
# Python 3.13 verifies strictly (VERIFY_X509_STRICT): the CA needs key usage
# and key identifiers, the leaf an authority key identifier.
openssl req -x509 -newkey rsa:2048 -nodes -days 3650 -subj "/CN=dotfiles-test CA" \
  -addext basicConstraints=critical,CA:TRUE -addext keyUsage=critical,keyCertSign,cRLSign \
  -addext subjectKeyIdentifier=hash -keyout ca.key -out ca.pem 2>/dev/null
openssl req -newkey rsa:2048 -nodes -subj "/CN=vw" -keyout key.pem -out leaf.csr 2>/dev/null
printf '%s\n' 'subjectAltName=DNS:vw,DNS:localhost' 'basicConstraints=CA:FALSE' \
  'extendedKeyUsage=serverAuth' 'keyUsage=critical,digitalSignature,keyEncipherment' \
  'subjectKeyIdentifier=hash' 'authorityKeyIdentifier=keyid' >ext.cnf
openssl x509 -req -in leaf.csr -CA ca.pem -CAkey ca.key -CAcreateserial -days 3650 \
  -extfile ext.cnf -out cert.pem 2>/dev/null
chmod 0644 ca.pem cert.pem key.pem
rm -f leaf.csr ext.cnf
echo "certificates written to $dir"
