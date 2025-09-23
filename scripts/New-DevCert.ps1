Write-Host "Creating a local dev code-signing cert (for test mode)..."
$cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject "CN=HIDra Dev" -CertStoreLocation Cert:\CurrentUser\My
$pwd = ConvertTo-SecureString -String "hidra-dev" -Force -AsPlainText
Export-PfxCertificate -Cert $cert -FilePath "$PSScriptRoot\hidra-dev.pfx" -Password $pwd
Write-Host "Created: $($cert.Thumbprint)"
