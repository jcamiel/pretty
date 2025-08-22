

Write-Host "Running pretty (iter=500):"
Measure-Command { [System.IO.File]::OpenRead("5mb.json") | .\target\release\pretty.exe --iter 500 | Out-Null }

Write-Host ""
Write-Host "Running pretty --no-color (iter=500):"
Measure-Command { [System.IO.File]::OpenRead("5mb.json") | .\target\release\pretty.exe --iter 500 --no-color | Out-Null }

Write-Host ""
Write-Host "Running pretty pretty --serde (iter=500):"
Measure-Command { [System.IO.File]::OpenRead("5mb.json") | .\target\release\pretty.exe --iter 500 --serde | Out-Null }
