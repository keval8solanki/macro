#!/bin/bash
set -e

APP_NAME="Macro"
CERT_NAME="$APP_NAME Self-Signed Cert"

echo "Generating Self-Signed Certificate for '$APP_NAME'..."

# 1. Create a config file for OpenSSL to ensure we have Code Signing extensions
cat > cert_config.cnf <<EOF
[ req ]
distinguished_name = req_distinguished_name
x509_extensions = v3_req
prompt = no

[ req_distinguished_name ]
CN = $CERT_NAME
O = $APP_NAME

[ v3_req ]
keyUsage = critical, digitalSignature
extendedKeyUsage = codeSigning
basicConstraints = critical, CA:FALSE
EOF

# 2. Generate Private Key and Certificate
# We use openssl which is standard on macOS
openssl req -x509 -newkey rsa:2048 -days 3650 -nodes \
  -keyout private.key \
  -out certificate.crt \
  -config cert_config.cnf

echo "Exporting certificate..."
echo "--------------------------------------------------------------------------------"
echo "🔐 You will now be asked to create a password for the export."
echo "   You MUST remember this password to put in the GitHub Secret 'MACOS_CERTIFICATE_PWD'."
echo "--------------------------------------------------------------------------------"

# 3. Export to .p12
openssl pkcs12 -export \
  -out macro_cert.p12 \
  -inkey private.key \
  -in certificate.crt \
  -name "$CERT_NAME"

# Cleanup
rm private.key certificate.crt cert_config.cnf

# Encode for GitHub
BASE64_CERT=$(base64 -i macro_cert.p12)

echo ""
echo "✅ Certificate generated: macro_cert.p12"
echo ""
echo "👉 ACTION REQUIRED: Add these secrets to your GitHub Repository:"
echo "   (Settings -> Secrets and variables -> Actions -> New repository secret)"
echo ""
echo "1. Name: MACOS_CERTIFICATE"
echo "   Value: (Copy the block below)"
echo "--------------------------------------------------------------------------------"
echo "$BASE64_CERT"
echo "--------------------------------------------------------------------------------"
echo ""
echo "2. Name: MACOS_CERTIFICATE_PWD"
echo "   Value: (The password you just typed)"
echo ""
echo "3. Name: KEYCHAIN_PASSWORD"
echo "   Value: $APP_NAME-build-password"
echo ""
echo "Done!"
