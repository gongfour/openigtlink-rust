#!/bin/bash
# Generate self-signed certificates for TLS testing
#
# Usage: ./examples/generate_test_certs.sh

set -e

CERT_DIR="examples/certs"
mkdir -p "$CERT_DIR"

echo "Generating test certificates in $CERT_DIR..."

# Generate private key
openssl genrsa -out "$CERT_DIR/key.pem" 2048

# Generate self-signed certificate (valid for 365 days)
openssl req -new -x509 -key "$CERT_DIR/key.pem" -out "$CERT_DIR/cert.pem" -days 365 \
    -subj "/C=US/ST=State/L=City/O=Organization/CN=localhost"

echo "✓ Certificate generated: $CERT_DIR/cert.pem"
echo "✓ Private key generated: $CERT_DIR/key.pem"
echo ""
echo "These are self-signed certificates for testing only."
echo "Do NOT use in production!"
