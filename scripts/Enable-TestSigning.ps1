Write-Host "Enabling Windows Test Signing and rebooting is usually required for unsigned drivers."
bcdedit /set testsigning on
Write-Host "=> Reboot to complete. To disable: bcdedit /set testsigning off"
