

Write-Host "pretty: "
Measure-Command { Get-Content 5mb.json -Raw | .\target\release\pretty.exe --iter 500 | Out-Null }

Write-Host ""
Write-Host "pretty --no-color:"
Measure-Command { Get-Content 5mb.json -Raw | .\target\release\pretty.exe --iter 500 --no-color | Out-Null }

Write-Host ""
Write-Host "pretty --serde:"
Measure-Command { Get-Content 5mb.json -Raw | .\target\release\pretty.exe --iter 500 --serde | Out-Null }
